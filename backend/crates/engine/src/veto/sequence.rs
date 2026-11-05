//! A ordem dos passos: o veto de mapas da série (modo veto) e o draft de
//! campeões. No veto de mapas, os lados alternam a cada passo, começando pelo
//! seed melhor (quais passos são bans e quais são picks vem de `VetoOrder`);
//! no draft, ver `draft_sequence`.

use serde::{Deserialize, Serialize};

use super::{VetoError, VetoOrder, other};
use crate::series::{Side, even_series};

/// O que um passo faz.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Tira um mapa do pool (veto) ou um campeão da série, para os dois.
    Ban,
    /// Escolhe o mapa de uma partida (veto) ou, no duelo alternado, o próprio
    /// campeão de uma partida.
    Pick,
    /// Vota num dos mapas oferecidos (modo votação).
    Vote,
    /// Um jogador escolhe o próprio campeão de uma partida (`PickOrder`): na
    /// janela simultânea, todos ao mesmo tempo; na ordem alternada, um por
    /// vez, quem for do lado da vez.
    TeamPick,
}

/// Os passos do veto de mapas de uma melhor de `best_of` com um pool de
/// `pool` mapas, sem o decisivo (o mapa que sobra, que é a última partida).
/// Ímpar:
///
/// Sobram `pool - best_of` bans e `best_of - 1` picks. `BansFirst`: todos os
/// bans, depois os picks. `Alternating`: dois bans primeiro (se houver),
/// depois os picks e por fim o resto dos bans. Com 7 mapas: md1, seis bans
/// nos dois; md3, quatro bans e dois picks (`BansFirst`) ou ban, ban, pick,
/// pick, ban, ban (`Alternating`); md5, ban, ban e quatro picks nos dois;
/// md7, seis picks. Na série par, só bans, até sobrarem N mapas (com 7, cinco na
/// md2, três na md4 e um na md6), nas duas ordens: não há pick nem decisivo.
/// Erro: `PoolTooSmall` com menos mapas que partidas.
pub fn map_sequence(pool: usize, best_of: u8, order: VetoOrder) -> Result<Vec<Action>, VetoError> {
    let games = usize::from(best_of);
    if best_of == 0 || pool < games {
        return Err(VetoError::PoolTooSmall { pool, best_of });
    }
    let bans = pool - games;
    if even_series(best_of) {
        return Ok(vec![Action::Ban; bans]);
    }
    let first = match order {
        VetoOrder::BansFirst => bans,
        VetoOrder::Alternating => bans.min(2),
    };
    let mut steps = vec![Action::Ban; first];
    steps.extend(std::iter::repeat_n(Action::Pick, games - 1));
    steps.extend(std::iter::repeat_n(Action::Ban, bans - first));
    Ok(steps)
}

/// Um passo do draft de campeões.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DraftStep {
    pub action: Action,
    pub side: Side,
    /// No pick, a partida a que ele se destina.
    pub game: Option<u8>,
}

/// Os passos do draft de campeões de uma melhor de `best_of`, com `bans` de
/// cada lado; `first` é o seed melhor, que começou o veto de mapas.
///
/// Primeiro os bans da série, alternados, começando pelo seed pior. No
/// duelo alternado, depois, um pick de cada lado por partida, na ordem das
/// partidas: nas ímpares escolhe primeiro o seed melhor e nas pares o pior, e
/// quem escolhe por último numa partida vê o campeão do outro. Num md3 com 1
/// ban: pior bane, melhor bane; partida 1, melhor e pior; partida 2, pior e
/// melhor; partida 3, melhor e pior. Com o pick por jogador (`per_player`),
/// só os bans: a ordem depende do elenco (`Action::TeamPick`, ver `team.rs`).
pub fn draft_sequence(bans: u8, best_of: u8, first: Side, per_player: bool) -> Vec<DraftStep> {
    let worse = other(first);
    let mut steps: Vec<DraftStep> = (0..usize::from(bans) * 2)
        .map(|i| DraftStep {
            action: Action::Ban,
            side: if i % 2 == 0 { worse } else { first },
            game: None,
        })
        .collect();
    if !per_player {
        for game in 1..=best_of {
            let opener = game_opener(first, game);
            for side in [opener, other(opener)] {
                steps.push(DraftStep {
                    action: Action::Pick,
                    side,
                    game: Some(game),
                });
            }
        }
    }
    steps
}

/// Quem abre as escolhas da partida `game`: nas ímpares o seed melhor
/// (`first`), nas pares o outro.
pub(super) const fn game_opener(first: Side, game: u8) -> Side {
    if game % 2 == 1 { first } else { other(first) }
}

/// O lado da posição `i` (a partir de 0) nas escolhas de uma partida que
/// `opener` abre: O, X, X, O, O, X, X, O... Com um jogador por lado, O, X.
pub(super) const fn pick_side(opener: Side, i: usize) -> Side {
    if i.div_ceil(2).is_multiple_of(2) {
        opener
    } else {
        other(opener)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Action::{Ban, Pick};
    use Side::{A, B};
    use VetoOrder::{Alternating, BansFirst};

    #[test]
    fn the_sequences_with_seven_maps() {
        assert_eq!(map_sequence(7, 1, Alternating).unwrap(), [Ban; 6]);
        assert_eq!(
            map_sequence(7, 3, Alternating).unwrap(),
            [Ban, Ban, Pick, Pick, Ban, Ban]
        );
        assert_eq!(
            map_sequence(7, 5, Alternating).unwrap(),
            [Ban, Ban, Pick, Pick, Pick, Pick]
        );
        assert_eq!(map_sequence(7, 7, Alternating).unwrap(), [Pick; 6]);
    }

    #[test]
    fn bans_first_puts_every_ban_before_the_picks() {
        assert_eq!(
            map_sequence(7, 3, BansFirst).unwrap(),
            [Ban, Ban, Ban, Ban, Pick, Pick]
        );
        assert_eq!(
            map_sequence(9, 3, BansFirst).unwrap(),
            [Ban, Ban, Ban, Ban, Ban, Ban, Pick, Pick]
        );
        assert_eq!(
            map_sequence(5, 3, BansFirst).unwrap(),
            [Ban, Ban, Pick, Pick]
        );
    }

    #[test]
    fn with_seven_maps_md1_and_md5_come_out_the_same_in_both_orders() {
        for best_of in [1, 5, 7] {
            assert_eq!(
                map_sequence(7, best_of, BansFirst),
                map_sequence(7, best_of, Alternating),
                "md{best_of}"
            );
        }
    }

    #[test]
    fn other_pools_keep_the_shape() {
        assert_eq!(
            map_sequence(5, 3, Alternating).unwrap(),
            [Ban, Ban, Pick, Pick]
        );
        assert_eq!(map_sequence(4, 3, Alternating).unwrap(), [Ban, Pick, Pick]);
        assert_eq!(map_sequence(1, 1, Alternating).unwrap(), []);
        assert_eq!(
            map_sequence(9, 3, Alternating).unwrap(),
            [Ban, Ban, Pick, Pick, Ban, Ban, Ban, Ban]
        );
        // Sempre sobra um mapa, o decisivo, e há um pick por partida menos uma.
        for order in [BansFirst, Alternating] {
            for pool in 1..=16 {
                for best_of in (1..=pool as u8).step_by(2) {
                    let steps = map_sequence(pool, best_of, order).unwrap();
                    assert_eq!(steps.len(), pool - 1);
                    let picks = steps.iter().filter(|a| **a == Pick).count();
                    assert_eq!(picks, usize::from(best_of) - 1);
                }
            }
        }
    }

    #[test]
    fn more_games_than_maps() {
        for order in [BansFirst, Alternating] {
            assert_eq!(
                map_sequence(3, 5, order),
                Err(VetoError::PoolTooSmall {
                    pool: 3,
                    best_of: 5
                })
            );
        }
    }

    #[test]
    fn an_even_series_only_bans_until_n_are_left() {
        assert_eq!(map_sequence(7, 2, BansFirst).unwrap(), [Ban; 5]);
        assert_eq!(map_sequence(7, 4, BansFirst).unwrap(), [Ban; 3]);
        assert_eq!(map_sequence(7, 6, BansFirst).unwrap(), [Ban; 1]);
        for best_of in [2u8, 4, 6] {
            let games = usize::from(best_of);
            for pool in games..=16 {
                let bans = map_sequence(pool, best_of, BansFirst).unwrap();
                assert_eq!(bans, vec![Ban; pool - games], "md{best_of}, pool {pool}");
                // A ordem do veto não faz diferença nestas séries.
                assert_eq!(map_sequence(pool, best_of, Alternating).unwrap(), bans);
            }
            assert_eq!(map_sequence(games, best_of, BansFirst).unwrap(), []);
            assert_eq!(
                map_sequence(games - 1, best_of, BansFirst),
                Err(VetoError::PoolTooSmall {
                    pool: games - 1,
                    best_of
                })
            );
        }
    }

    /// O draft como (ação, lado, partida), com A no seed melhor.
    fn draft(bans: u8, best_of: u8, teams: bool) -> Vec<(Action, Side, Option<u8>)> {
        draft_sequence(bans, best_of, A, teams)
            .into_iter()
            .map(|s| (s.action, s.side, s.game))
            .collect()
    }

    #[test]
    fn duel_md3_with_one_ban_each() {
        assert_eq!(
            draft(1, 3, false),
            [
                (Ban, B, None),
                (Ban, A, None),
                (Pick, A, Some(1)),
                (Pick, B, Some(1)),
                (Pick, B, Some(2)),
                (Pick, A, Some(2)),
                (Pick, A, Some(3)),
                (Pick, B, Some(3)),
            ]
        );
    }

    #[test]
    fn every_length_gives_each_side_its_bans_and_a_pick_per_game() {
        for best_of in [1u8, 2, 3, 4, 5, 6, 7] {
            for bans in 0..=3u8 {
                let steps = draft(bans, best_of, false);
                assert_eq!(
                    steps.len(),
                    usize::from(bans) * 2 + usize::from(best_of) * 2
                );
                for side in [A, B] {
                    let mine = |a: Action| steps.iter().filter(|s| s.0 == a && s.1 == side).count();
                    assert_eq!(mine(Ban), usize::from(bans));
                    assert_eq!(mine(Pick), usize::from(best_of));
                }
                // Os bans vêm antes dos picks; os picks, partida a partida.
                let games: Vec<u8> = steps.iter().filter_map(|s| s.2).collect();
                assert!(games.windows(2).all(|w| w[0] <= w[1]));
                assert!(steps[..usize::from(bans) * 2].iter().all(|s| s.0 == Ban));
                // Quem escolhe por último alterna: o seed pior nas ímpares.
                for game in 1..=best_of {
                    let last = steps.iter().rev().find(|s| s.2 == Some(game)).unwrap();
                    assert_eq!(last.1, if game % 2 == 1 { B } else { A }, "partida {game}");
                }
            }
        }
    }

    #[test]
    fn md1_and_md7() {
        assert_eq!(
            draft(2, 1, false),
            [
                (Ban, B, None),
                (Ban, A, None),
                (Ban, B, None),
                (Ban, A, None),
                (Pick, A, Some(1)),
                (Pick, B, Some(1)),
            ]
        );
        let md7 = draft(0, 7, false);
        assert_eq!(md7.len(), 14);
        assert_eq!(md7[12..], [(Pick, A, Some(7)), (Pick, B, Some(7))]);
    }

    #[test]
    fn picks_per_player_only_ban() {
        assert_eq!(draft(1, 3, true), [(Ban, B, None), (Ban, A, None)]);
        assert!(draft(0, 5, true).is_empty());
        // Com B no seed melhor, A bane primeiro.
        let steps = draft_sequence(1, 1, B, false);
        assert_eq!(steps[0].side, A);
        assert_eq!(steps[2].side, B);
    }

    #[test]
    fn the_pick_pattern_by_team_size() {
        let pattern =
            |size: usize| -> Vec<Side> { (0..2 * size).map(|i| pick_side(A, i)).collect() };
        assert_eq!(pattern(1), [A, B]);
        assert_eq!(pattern(2), [A, B, B, A]);
        assert_eq!(pattern(3), [A, B, B, A, A, B]);
        assert_eq!(pattern(4), [A, B, B, A, A, B, B, A]);
    }
}
