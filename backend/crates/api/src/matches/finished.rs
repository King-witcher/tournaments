//! O fim do torneio, um ponto só para os dois caminhos que o encerram (a chave
//! que termina, em `changes::settled`, e as fases que faltavam removidas, na
//! edição): o pódio no Discord (`announce`) e o aviso de quem jogou, no sininho
//! e no Web Push, na mesma transação.
//!
//! Recebe o aviso quem tem colocação na classificação final (`chain::standings`,
//! com os cortados e os ausentes das fases anteriores), menos o time que
//! desistiu e quem saiu da comunidade: `tournament.won` no duelo,
//! `tournament.team_won` para os jogadores do time campeão e, para os demais,
//! `tournament.placed` com o campeão e a posição. Uma correção que reabre o
//! torneio faz o encerramento seguinte avisar de novo, com o resultado certo.
//!
//! O 1º lugar nunca é dividido (a final tem sempre vencedor, a tabela dá uma
//! posição a cada um e não termina com empate esperando a organização): o
//! campeão é sempre um só.

use engine::core::Placement;
use sqlx::PgConnection;

use super::{announce, chain, stage};
use crate::{
    outbox::{self, Event, Notice, NoticeKind},
    shared::ApiError,
    teams::StoredTeam,
    tournaments::{Address, StoredTournament},
};

/// O torneio acabou de encerrar, com `stage` já lida: o pódio e os avisos.
pub async fn finished(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &stage::Stage,
) -> Result<(), ApiError> {
    let standings = chain::standings(&stage::chain(conn, tournament, stage).await?);
    let audience = tournament.address();
    announce::finished(conn, &audience, stage, &standings).await?;
    let events = notices(
        &audience,
        tournament.setup.teams(),
        &stage.teams,
        &standings,
    );
    outbox::emit_all(conn, events).await?;
    Ok(())
}

/// O torneio encerrou fora da chave (as fases que faltavam saíram, com a atual
/// terminada): o mesmo que no fim pela chave.
pub async fn announce_finished(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
) -> Result<(), ApiError> {
    let stage = stage::load_current(conn, tournament).await?;
    finished(conn, tournament, &stage).await
}

/// Um aviso por time com colocação, para os jogadores dele que continuam na
/// comunidade. `in_teams`: TDM (o campeão vai pelo nome do time).
fn notices(
    audience: &Address,
    in_teams: bool,
    teams: &[StoredTeam],
    standings: &[Placement],
) -> Vec<Event> {
    let team = |p: &Placement| teams.iter().find(|t| t.id == p.participant.0);
    let Some(champion) = standings.iter().find(|p| p.rank == 1).and_then(team) else {
        return Vec::new();
    };
    standings
        .iter()
        .filter_map(|p| {
            let to = team(p).filter(|t| !t.out())?;
            let tournament = audience.name.clone();
            let kind = if to.id != champion.id {
                NoticeKind::TournamentPlaced {
                    tournament,
                    champion: champion.display_name(),
                    position: position(p),
                }
            } else if in_teams {
                NoticeKind::TournamentTeamWon {
                    tournament,
                    team: to.display_name(),
                }
            } else {
                NoticeKind::TournamentWon { tournament }
            };
            let notice = Notice {
                kind,
                path: audience.path(),
            };
            let players = to.members.iter().filter(|m| !m.left).map(|m| m.user_id);
            Some(Event::notice(players, notice))
        })
        .collect()
}

/// A posição pronta para o texto, igual em todo idioma: `5` ou, numa faixa de
/// empate da eliminação, `5–8`.
fn position(p: &Placement) -> String {
    if p.tied > 1 {
        format!("{}–{}", p.rank, p.rank + p.tied - 1)
    } else {
        p.rank.to_string()
    }
}

#[cfg(test)]
mod tests {
    use engine::core::ParticipantId;

    use super::*;

    #[test]
    fn a_tied_position_is_a_range() {
        let at = |rank, tied| Placement {
            participant: ParticipantId(1),
            rank,
            tied,
        };
        assert_eq!(position(&at(2, 1)), "2");
        assert_eq!(position(&at(5, 4)), "5–8");
    }
}
