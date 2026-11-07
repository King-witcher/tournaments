//! O rating de cada time congelado no início, para o desempate pelo menor
//! rating (`docs/decisions/formats.md`, "Desempate por fase"). O rating da
//! comunidade muda a cada mapa e correção; a tabela precisa sair igual em todo
//! replay, então vale o do início, gravado em `teams.rating_at_start`. Só nos
//! torneios com o menor rating entre os desempates ou com mais de uma fase (as
//! que não começaram podem passar a usá-lo); desfazer o início o apaga, e
//! iniciar de novo grava outra vez. A substituição não muda. Todas as fases
//! usam o do início do torneio.

use engine::formats::table::Criterion;
use sqlx::PgConnection;

use crate::{ratings, shared::ApiError, teams, tournaments::StoredTournament};

/// Grava o rating do início dos times da chave (já com os seeds do início),
/// se alguma fase usa o menor rating: a média dos jogadores, com o rating da
/// comunidade de agora (recalculado antes, se há marca) e o inicial para quem
/// não tem, mesmo em torneio que não conta para o rating.
pub async fn freeze(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
) -> Result<(), ApiError> {
    // Com mais de uma fase, sempre: as que não começaram ainda podem passar a
    // usá-lo (a configuração delas muda até começarem), e o de então é o do
    // início do torneio.
    let uses = tournament.setup.stages.len() > 1
        || tournament.setup.stages.iter().any(|s| {
            s.format
                .tiebreakers()
                .is_some_and(|t| t.contains(Criterion::LowerRating))
        });
    if !uses {
        return Ok(());
    }
    let entrants = teams::seeded(conn, tournament.id).await?;
    let users: Vec<i64> = entrants.iter().flat_map(|t| t.user_ids()).collect();
    let current = ratings::current(conn, tournament.tenant, &users).await?;
    let frozen: Vec<(i64, i32)> = entrants
        .iter()
        .filter_map(|t| {
            let players: Vec<i32> = t
                .user_ids()
                .iter()
                .filter_map(|u| current.get(u).copied())
                .collect();
            average_centi(&players).map(|r| (t.id, r))
        })
        .collect();
    teams::save_start_ratings(conn, tournament.id, &frozen).await?;
    Ok(())
}

/// A média de `ratings` em centésimos, arredondada (a metade para cima):
/// um inteiro, para o desempate comparar sem `f64`. Com até 4 jogadores por
/// time, médias diferentes diferem em pelo menos 1/12 e nunca dão o mesmo
/// valor. `None` sem jogador.
fn average_centi(ratings: &[i32]) -> Option<i32> {
    let n = i64::try_from(ratings.len()).ok().filter(|n| *n > 0)?;
    let sum: i64 = ratings.iter().map(|r| i64::from(*r)).sum::<i64>() * 100;
    let (q, r) = (sum.div_euclid(n), sum.rem_euclid(n));
    let rounded = if 2 * r >= n { q + 1 } else { q };
    i32::try_from(rounded).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn average_in_hundredths() {
        assert_eq!(average_centi(&[]), None);
        assert_eq!(average_centi(&[1500]), Some(150_000));
        assert_eq!(average_centi(&[1500, 1501]), Some(150_050));
        // 4501 / 3 = 1500,333...; 4502 / 3 = 1500,666...
        assert_eq!(average_centi(&[1500, 1500, 1501]), Some(150_033));
        assert_eq!(average_centi(&[1500, 1501, 1501]), Some(150_067));
        // O Elo não tem piso.
        assert_eq!(average_centi(&[-1, -2]), Some(-150));
    }

    #[test]
    fn different_averages_never_meet() {
        // Times de 1 a 4 jogadores com somas perto umas das outras: a média
        // em centésimos separa toda média diferente.
        let mut seen: Vec<(i64, i64, i32)> = Vec::new();
        for n in 1..=4i64 {
            for sum in 6000..6020i64 {
                let mut players = vec![1500i32; n as usize - 1];
                players.push(i32::try_from(sum - 1500 * (n - 1)).unwrap());
                seen.push((sum, n, average_centi(&players).unwrap()));
            }
        }
        for &(s1, n1, a1) in &seen {
            for &(s2, n2, a2) in &seen {
                if s1 * n2 != s2 * n1 {
                    assert_ne!(a1, a2, "{s1}/{n1} e {s2}/{n2}");
                }
            }
        }
    }
}
