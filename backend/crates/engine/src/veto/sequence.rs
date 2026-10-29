//! A ordem dos passos: o veto de mapas da série (modo veto) e o ban/pick de
//! campeões de cada mapa. Os lados alternam a cada passo, começando por quem
//! começa (o seed melhor no veto de mapas; nos campeões, ver `Veto::starter`).

use serde::{Deserialize, Serialize};

use super::VetoError;

/// O que um passo faz.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Tira um mapa do pool (veto) ou um campeão do mapa, para os dois.
    Ban,
    /// Escolhe o mapa de um jogo (veto) ou o próprio campeão do mapa.
    Pick,
    /// Vota num dos mapas oferecidos (modo votação).
    Vote,
}

/// Os passos do veto de mapas de uma melhor de `best_of` com um pool de
/// `pool` mapas, sem o decisivo (o mapa que sobra, que é o último jogo).
///
/// Sobram `pool - best_of` bans e `best_of - 1` picks: dois bans primeiro
/// (se houver), depois os picks e por fim o resto dos bans. Com 7 mapas:
/// md1, seis bans; md3, ban, ban, pick, pick, ban, ban; md5, ban, ban e
/// quatro picks; md7, seis picks. Erro: `PoolTooSmall` com menos mapas que
/// jogos.
pub fn map_sequence(pool: usize, best_of: u8) -> Result<Vec<Action>, VetoError> {
    let games = usize::from(best_of);
    if best_of == 0 || pool < games {
        return Err(VetoError::PoolTooSmall { pool, best_of });
    }
    let bans = pool - games;
    let first = bans.min(2);
    let mut steps = vec![Action::Ban; first];
    steps.extend(std::iter::repeat_n(Action::Pick, games - 1));
    steps.extend(std::iter::repeat_n(Action::Ban, bans - first));
    Ok(steps)
}

/// Os passos de campeões de um mapa: `bans` rodadas de um ban de cada lado e
/// depois um pick de cada lado.
pub fn champion_sequence(bans: u8) -> Vec<Action> {
    let mut steps = vec![Action::Ban; usize::from(bans) * 2];
    steps.extend([Action::Pick, Action::Pick]);
    steps
}

#[cfg(test)]
mod tests {
    use super::*;
    use Action::{Ban, Pick};

    #[test]
    fn the_sequences_with_seven_maps() {
        assert_eq!(map_sequence(7, 1).unwrap(), [Ban; 6]);
        assert_eq!(
            map_sequence(7, 3).unwrap(),
            [Ban, Ban, Pick, Pick, Ban, Ban]
        );
        assert_eq!(
            map_sequence(7, 5).unwrap(),
            [Ban, Ban, Pick, Pick, Pick, Pick]
        );
        assert_eq!(map_sequence(7, 7).unwrap(), [Pick; 6]);
    }

    #[test]
    fn other_pools_keep_the_shape() {
        assert_eq!(map_sequence(5, 3).unwrap(), [Ban, Ban, Pick, Pick]);
        assert_eq!(map_sequence(4, 3).unwrap(), [Ban, Pick, Pick]);
        assert_eq!(map_sequence(1, 1).unwrap(), []);
        assert_eq!(
            map_sequence(9, 3).unwrap(),
            [Ban, Ban, Pick, Pick, Ban, Ban, Ban, Ban]
        );
        // Sempre sobra um mapa, o decisivo.
        for pool in 1..=16 {
            for best_of in (1..=pool as u8).step_by(2) {
                assert_eq!(map_sequence(pool, best_of).unwrap().len(), pool - 1);
            }
        }
    }

    #[test]
    fn more_games_than_maps() {
        assert_eq!(
            map_sequence(3, 5),
            Err(VetoError::PoolTooSmall {
                pool: 3,
                best_of: 5
            })
        );
    }

    #[test]
    fn champions() {
        assert_eq!(champion_sequence(0), [Pick, Pick]);
        assert_eq!(champion_sequence(2), [Ban, Ban, Ban, Ban, Pick, Pick]);
    }
}
