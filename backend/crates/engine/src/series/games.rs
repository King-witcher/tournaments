//! A série mapa a mapa. Cada mapa é enviado e confirmado sozinho; a série
//! fecha quando os mapas confirmados, em sequência desde o primeiro, dão a
//! maioria a um lado (2 na md3, 3 na md5). A série par também (2 na md2, 3 na
//! md4, 4 na md6) e, sem ninguém na maioria, fecha com os N mapas confirmados,
//! empatada (`Decision::winner` nulo): 1 x 1, 2 x 2 ou 3 x 3.
//!
//! Regras:
//! - o mapa N+1 só vem depois do N (enviado, não precisa estar confirmado);
//! - não há mapa depois da decisão: o próximo é recusado quando os mapas não
//!   contestados já dão a maioria a alguém (`SeriesDecided`) e quando a série
//!   já teve os N mapas da melhor de N (`NoGamesLeft`);
//! - o mapa contestado não conta para essa decisão (o vencedor dele está em
//!   dúvida), então contestar não trava o envio do seguinte; quando a
//!   contestação se resolve (ou o organizador corrige um mapa), os mapas que
//!   ficaram depois da decisão saem (`SeriesState::standing`);
//! - o mapa não se repete na série; mapa nulo (resultado antigo, sem
//!   detalhe) não conta como repetido;
//! - o vencedor do mapa é quem fez mais frags, e empate não existe
//!   (`winner_from_frags`). Os frags podem ser negativos (suicídio e queda
//!   tiram frag): -1 vence -3.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use super::{even_series, wins_needed};
use crate::core::{EngineError, MapId};

/// Um dos dois lados da partida.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    A,
    B,
}

/// Estado de um mapa enviado.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameStatus {
    /// Enviado por um jogador, esperando o adversário (ou o fim da janela).
    Pending,
    /// O adversário contestou: espera o organizador.
    Contested,
    /// Vale: pelo adversário, pelo tempo ou pelo organizador.
    Confirmed,
}

/// Um mapa da série, na ordem jogada.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeriesGame {
    pub winner: Side,
    pub status: GameStatus,
    /// `None` só em resultado antigo, enviado sem detalhe por mapa.
    pub map: Option<MapId>,
}

/// A série decidida: o vencedor e os mapas de cada lado. Sem vencedor, a série
/// par empatada, com metade dos mapas para cada lado.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub winner: Option<Side>,
    pub a_wins: u8,
    pub b_wins: u8,
}

/// O que os mapas enviados dizem da série.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeriesState {
    /// Quantos mapas, do primeiro, continuam valendo. Os seguintes vieram
    /// depois da decisão pelos mapas não contestados e saem (anulados).
    pub standing: usize,
    /// Decidida pelos mapas confirmados: todos os mapas até o que deu a
    /// maioria estão confirmados (sem maioria, na série par, todos os N).
    pub decided: Option<Decision>,
}

#[derive(Default)]
struct Tally {
    a: u8,
    b: u8,
}

impl Tally {
    fn add(&mut self, side: Side) {
        match side {
            Side::A => self.a += 1,
            Side::B => self.b += 1,
        }
    }

    fn leader(&self, needed: u8) -> Option<Side> {
        if self.a >= needed {
            Some(Side::A)
        } else if self.b >= needed {
            Some(Side::B)
        } else {
            None
        }
    }
}

/// Onde a série está, numa melhor de `best_of`, com `games` na ordem.
pub fn evaluate(best_of: u8, games: &[SeriesGame]) -> SeriesState {
    let needed = wins_needed(best_of);
    let mut open = Tally::default();
    let mut standing = games.len().min(usize::from(best_of));
    for (i, g) in games.iter().enumerate().take(standing) {
        if g.status != GameStatus::Contested {
            open.add(g.winner);
        }
        if open.leader(needed).is_some() {
            standing = i + 1;
            break;
        }
    }

    let mut confirmed = Tally::default();
    let mut decided = None;
    for (i, g) in games[..standing].iter().enumerate() {
        if g.status != GameStatus::Confirmed {
            break;
        }
        confirmed.add(g.winner);
        let winner = confirmed.leader(needed);
        // Sem maioria, a série par só fecha com os N mapas: é o empate.
        if winner.is_some() || (even_series(best_of) && i + 1 == usize::from(best_of)) {
            decided = Some(Decision {
                winner,
                a_wins: confirmed.a,
                b_wins: confirmed.b,
            });
            break;
        }
    }
    SeriesState { standing, decided }
}

/// O próximo mapa (o número `games.len() + 1`) pode ser enviado, com `map`?
/// Erros: `SeriesDecided` (os mapas não contestados já dão a maioria),
/// `NoGamesLeft` (a série já teve os `best_of` mapas) e `MapRepeated`.
pub fn check_next(
    best_of: u8,
    games: &[SeriesGame],
    map: Option<MapId>,
) -> Result<(), EngineError> {
    let mut open = Tally::default();
    for g in games.iter().filter(|g| g.status != GameStatus::Contested) {
        open.add(g.winner);
    }
    if open.leader(wins_needed(best_of)).is_some() {
        return Err(EngineError::SeriesDecided);
    }
    if games.len() >= usize::from(best_of) {
        return Err(EngineError::NoGamesLeft { best_of });
    }
    check_map(games, None, map)
}

/// Quantos mapas ainda faltam, no mínimo, para a série acabar, contando o que
/// está em jogo: as vitórias que faltam a quem está na frente. Mapa
/// contestado não conta (como em `check_next`). Na série par, a que pode
/// acabar sem maioria, nunca mais que os mapas ainda não enviados.
pub fn games_left(best_of: u8, games: &[SeriesGame]) -> u8 {
    let mut open = Tally::default();
    for g in games.iter().filter(|g| g.status != GameStatus::Contested) {
        open.add(g.winner);
    }
    let wins = wins_needed(best_of).saturating_sub(open.a.max(open.b));
    if even_series(best_of) {
        wins.min(usize::from(best_of).saturating_sub(games.len()) as u8)
    } else {
        wins
    }
}

/// Grava `game` no índice `index` (a partir de 0) da série: o próximo mapa
/// (`index == games.len()`, com as regras de `check_next`) ou a troca de um
/// já enviado (resolução ou correção). Devolve onde a série fica. Na troca,
/// o mapa só não pode repetir os que continuam valendo depois dela (os que
/// passam a vir depois da decisão saem). Erros: `GameOutOfOrder`, os de
/// `check_next` e `MapRepeated`.
pub fn put(
    best_of: u8,
    games: &[SeriesGame],
    index: usize,
    game: SeriesGame,
) -> Result<SeriesState, EngineError> {
    let mut next = games.to_vec();
    if index == games.len() {
        check_next(best_of, games, game.map)?;
        next.push(game);
        return Ok(evaluate(best_of, &next));
    }
    let slot = next.get_mut(index).ok_or(EngineError::GameOutOfOrder {
        next: games.len() + 1,
    })?;
    *slot = game;
    let state = evaluate(best_of, &next);
    if index >= state.standing {
        return Err(EngineError::SeriesDecided);
    }
    check_map(&next[..state.standing], Some(index), game.map)?;
    Ok(state)
}

/// `map` não se repete na série, sem contar o mapa de índice `except` (o que
/// está sendo trocado). Mapa nulo não repete.
pub fn check_map(
    games: &[SeriesGame],
    except: Option<usize>,
    map: Option<MapId>,
) -> Result<(), EngineError> {
    let Some(map) = map else { return Ok(()) };
    let repeated = games
        .iter()
        .enumerate()
        .any(|(i, g)| Some(i) != except && g.map == Some(map));
    if repeated {
        return Err(EngineError::MapRepeated(map));
    }
    Ok(())
}

/// Quem venceu o mapa `game` (a partir de 1) com `a` frags para o lado `a` e
/// `b` para o lado `b`: quem fez mais, negativos inclusive (a faixa é do
/// site). Erro: `GameTied`, porque empate não tem vencedor.
pub fn winner_from_frags(game: usize, a: i16, b: i16) -> Result<Side, EngineError> {
    match a.cmp(&b) {
        Ordering::Greater => Ok(Side::A),
        Ordering::Less => Ok(Side::B),
        Ordering::Equal => Err(EngineError::GameTied { game }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use GameStatus::*;
    use Side::*;

    fn g(winner: Side, status: GameStatus) -> SeriesGame {
        SeriesGame {
            winner,
            status,
            map: None,
        }
    }

    fn on(map: i64, winner: Side) -> SeriesGame {
        SeriesGame {
            winner,
            status: Confirmed,
            map: Some(MapId(map)),
        }
    }

    fn decision(winner: Side, a_wins: u8, b_wins: u8) -> Option<Decision> {
        Some(Decision {
            winner: Some(winner),
            a_wins,
            b_wins,
        })
    }

    const fn drawn(maps: u8) -> Option<Decision> {
        Some(Decision {
            winner: None,
            a_wins: maps,
            b_wins: maps,
        })
    }

    const DRAW: Option<Decision> = drawn(1);

    #[test]
    fn the_series_closes_on_the_confirmed_majority() {
        let s = evaluate(3, &[g(A, Confirmed), g(B, Confirmed), g(A, Confirmed)]);
        assert_eq!((s.standing, s.decided), (3, decision(A, 2, 1)));
        let s = evaluate(5, &[g(B, Confirmed), g(B, Confirmed), g(B, Confirmed)]);
        assert_eq!((s.standing, s.decided), (3, decision(B, 0, 3)));
        assert_eq!(evaluate(1, &[g(B, Confirmed)]).decided, decision(B, 0, 1));
    }

    #[test]
    fn open_until_every_map_up_to_the_majority_is_confirmed() {
        assert_eq!(evaluate(3, &[]).decided, None);
        assert_eq!(evaluate(3, &[g(A, Confirmed)]).decided, None);
        // O segundo mapa ainda espera o adversário.
        assert_eq!(evaluate(3, &[g(A, Confirmed), g(A, Pending)]).decided, None);
        // O primeiro está contestado: o placar ainda pode mudar.
        let s = evaluate(3, &[g(A, Contested), g(B, Confirmed), g(B, Confirmed)]);
        assert_eq!((s.standing, s.decided), (3, None));
    }

    #[test]
    fn maps_after_the_decision_do_not_stand() {
        // Resolvida a contestação a favor de A, o terceiro mapa não existiria.
        let s = evaluate(3, &[g(A, Confirmed), g(A, Confirmed), g(B, Pending)]);
        assert_eq!((s.standing, s.decided), (2, decision(A, 2, 0)));
        // Pendentes também decidem onde a série acaba (ainda não fecha).
        let s = evaluate(3, &[g(A, Pending), g(A, Pending), g(B, Confirmed)]);
        assert_eq!((s.standing, s.decided), (2, None));
        // O contestado não conta: o que veio depois continua valendo.
        let s = evaluate(3, &[g(A, Contested), g(A, Confirmed), g(B, Confirmed)]);
        assert_eq!((s.standing, s.decided), (3, None));
    }

    #[test]
    fn the_next_map_waits_for_nothing_but_the_previous_one() {
        assert_eq!(check_next(3, &[], None), Ok(()));
        // O anterior só precisa ter sido enviado.
        assert_eq!(check_next(3, &[g(A, Pending)], None), Ok(()));
        // Contestar não trava o seguinte, nem quando o contestado decidiria.
        assert_eq!(
            check_next(3, &[g(A, Pending), g(A, Contested)], None),
            Ok(())
        );
    }

    #[test]
    fn games_left_counts_the_wins_the_leader_still_needs() {
        assert_eq!(games_left(3, &[]), 2);
        assert_eq!(games_left(3, &[g(A, Pending)]), 1);
        assert_eq!(games_left(3, &[g(A, Confirmed), g(B, Confirmed)]), 1);
        assert_eq!(games_left(1, &[]), 1);
        assert_eq!(games_left(5, &[g(A, Confirmed), g(A, Pending)]), 1);
        assert_eq!(games_left(3, &[g(A, Contested)]), 2);
        assert_eq!(games_left(3, &[g(B, Confirmed), g(B, Confirmed)]), 0);
        // Passou da maioria (mapas que vão sair): satura em 0.
        assert_eq!(games_left(1, &[g(A, Pending), g(A, Pending)]), 0);
    }

    #[test]
    fn no_map_after_the_decision() {
        let decided = Err(EngineError::SeriesDecided);
        assert_eq!(
            check_next(3, &[g(A, Confirmed), g(A, Confirmed)], None),
            decided
        );
        // Pelos enviados, ainda sem confirmar.
        assert_eq!(
            check_next(3, &[g(B, Pending), g(B, Pending)], None),
            decided
        );
        assert_eq!(check_next(1, &[g(A, Pending)], None), decided);
        // Com um contestado, a md3 pode chegar aos três mapas sem decisão.
        let full = [g(A, Contested), g(A, Confirmed), g(B, Confirmed)];
        assert_eq!(
            check_next(3, &full, None),
            Err(EngineError::NoGamesLeft { best_of: 3 })
        );
    }

    #[test]
    fn maps_do_not_repeat() {
        let games = [on(1, A), on(2, B)];
        assert_eq!(
            check_next(3, &games, Some(MapId(2))),
            Err(EngineError::MapRepeated(MapId(2)))
        );
        assert_eq!(check_next(3, &games, Some(MapId(3))), Ok(()));
        // Trocar um mapa pelo mesmo não é repetir.
        assert_eq!(check_map(&games, Some(1), Some(MapId(2))), Ok(()));
        assert_eq!(
            check_map(&games, Some(0), Some(MapId(2))),
            Err(EngineError::MapRepeated(MapId(2)))
        );
        // Mapa nulo (resultado antigo) não repete.
        assert_eq!(check_map(&[g(A, Confirmed)], None, None), Ok(()));
    }

    #[test]
    fn put_appends_or_replaces() {
        let games = [on(1, A), on(2, B), on(3, A)];
        let next = |winner| SeriesGame {
            winner,
            status: Confirmed,
            map: Some(MapId(3)),
        };
        assert_eq!(
            put(3, &games[..1], 3, next(A)),
            Err(EngineError::GameOutOfOrder { next: 2 })
        );
        assert_eq!(
            put(3, &games[..2], 2, next(A)).map(|s| s.decided),
            Ok(decision(A, 2, 1))
        );
        // Corrigir o mapa 2 para A decide no segundo: o terceiro sai, e o
        // mapa dele pode ir para o segundo.
        let fixed = put(3, &games, 1, next(A)).unwrap();
        assert_eq!((fixed.standing, fixed.decided), (2, decision(A, 2, 0)));
        // Trocar o vencedor do primeiro: B decide no segundo.
        let flipped = put(3, &games, 0, on(1, B)).unwrap();
        assert_eq!((flipped.standing, flipped.decided), (2, decision(B, 0, 2)));
        // Numa série 2 x 0, trocar o primeiro a reabre: 1 x 1.
        let reopened = put(3, &[on(1, A), on(2, A)], 0, on(1, B)).unwrap();
        assert_eq!((reopened.standing, reopened.decided), (2, None));
        assert_eq!(
            put(3, &games, 0, on(2, A)),
            Err(EngineError::MapRepeated(MapId(2)))
        );
        assert_eq!(put(3, &games, 3, on(4, B)), Err(EngineError::SeriesDecided));
    }

    #[test]
    fn best_of_two_closes_only_with_both_maps_confirmed() {
        let closed = |games: &[SeriesGame]| evaluate(2, games).decided;
        assert_eq!(
            closed(&[g(A, Confirmed), g(A, Confirmed)]),
            decision(A, 2, 0)
        );
        assert_eq!(
            closed(&[g(B, Confirmed), g(B, Confirmed)]),
            decision(B, 0, 2)
        );
        assert_eq!(closed(&[g(A, Confirmed), g(B, Confirmed)]), DRAW);
        assert_eq!(closed(&[g(B, Confirmed), g(A, Confirmed)]), DRAW);
        // Um mapa só nunca decide, nem por W.O. de ninguém: falta o outro.
        assert_eq!(closed(&[]), None);
        assert_eq!(closed(&[g(A, Confirmed)]), None);
        // Em sequência: o primeiro sem confirmar segura mesmo com o segundo.
        assert_eq!(closed(&[g(A, Pending), g(A, Confirmed)]), None);
        assert_eq!(closed(&[g(A, Confirmed), g(B, Pending)]), None);
        assert_eq!(closed(&[g(A, Contested), g(B, Confirmed)]), None);
        assert_eq!(closed(&[g(A, Confirmed), g(B, Contested)]), None);
    }

    #[test]
    fn best_of_two_always_plays_both_maps() {
        // O primeiro mapa não decide, em nenhum estado: o segundo pode vir.
        for status in [Pending, Contested, Confirmed] {
            assert_eq!(check_next(2, &[g(A, status)], None), Ok(()));
        }
        // Depois dos dois, nada: 2 x 0 está decidido e 1 x 1 esgotou os mapas.
        assert_eq!(
            check_next(2, &[g(A, Pending), g(A, Pending)], None),
            Err(EngineError::SeriesDecided)
        );
        let full = Err(EngineError::NoGamesLeft { best_of: 2 });
        assert_eq!(check_next(2, &[g(A, Confirmed), g(B, Pending)], None), full);
        assert_eq!(check_next(2, &[g(A, Contested), g(A, Pending)], None), full);
        // Nada fica fora do que vale.
        let s = evaluate(2, &[g(A, Confirmed), g(B, Confirmed), g(A, Pending)]);
        assert_eq!((s.standing, s.decided), (2, DRAW));
    }

    #[test]
    fn best_of_two_counts_the_maps_not_yet_sent() {
        assert_eq!(games_left(2, &[]), 2);
        assert_eq!(games_left(2, &[g(A, Pending)]), 1);
        assert_eq!(games_left(2, &[g(A, Contested)]), 1);
        assert_eq!(games_left(2, &[g(A, Confirmed), g(B, Confirmed)]), 0);
        assert_eq!(games_left(2, &[g(A, Confirmed), g(A, Confirmed)]), 0);
    }

    #[test]
    fn correcting_a_best_of_two_moves_between_win_and_draw() {
        let won = [on(1, A), on(2, A)];
        assert_eq!(put(2, &won, 1, on(2, B)).unwrap().decided, DRAW);
        let drawn = [on(1, A), on(2, B)];
        let fixed = put(2, &drawn, 1, on(2, A)).unwrap();
        assert_eq!((fixed.standing, fixed.decided), (2, decision(A, 2, 0)));
        let flipped = put(2, &drawn, 0, on(1, B)).unwrap();
        assert_eq!((flipped.standing, flipped.decided), (2, decision(B, 0, 2)));
        // O mapa do outro índice continua sem repetir.
        assert_eq!(
            put(2, &drawn, 0, on(2, A)),
            Err(EngineError::MapRepeated(MapId(2)))
        );
    }

    /// A série com estes vencedores, todos confirmados.
    fn confirmed(winners: &[Side]) -> Vec<SeriesGame> {
        winners.iter().map(|w| g(*w, Confirmed)).collect()
    }

    #[test]
    fn best_of_four_ends_at_the_majority_or_tied_at_two() {
        let closed = |w: &[Side]| evaluate(4, &confirmed(w)).decided;
        assert_eq!(closed(&[A, A, A]), decision(A, 3, 0));
        assert_eq!(closed(&[B, A, B, B]), decision(B, 1, 3));
        assert_eq!(closed(&[A, B, A, A]), decision(A, 3, 1));
        assert_eq!(closed(&[A, B, A, B]), drawn(2));
        assert_eq!(closed(&[B, B, A, A]), drawn(2));
        // Sem maioria e sem os 4 mapas, a série continua.
        assert_eq!(closed(&[A, A]), None);
        assert_eq!(closed(&[A, B, A]), None);
        assert_eq!(
            evaluate(
                4,
                &[
                    g(A, Confirmed),
                    g(B, Confirmed),
                    g(A, Confirmed),
                    g(B, Pending)
                ]
            )
            .decided,
            None
        );
    }

    #[test]
    fn best_of_six_ends_at_the_majority_or_tied_at_three() {
        let closed = |w: &[Side]| evaluate(6, &confirmed(w)).decided;
        assert_eq!(closed(&[A, A, A, A]), decision(A, 4, 0));
        assert_eq!(closed(&[A, B, A, A, A]), decision(A, 4, 1));
        assert_eq!(closed(&[B, B, A, B, A, B]), decision(B, 2, 4));
        assert_eq!(closed(&[A, B, A, B, A, B]), drawn(3));
        assert_eq!(closed(&[A, A, A, B, B]), None);
    }

    #[test]
    fn no_map_after_the_majority_in_an_even_series() {
        let decided = Err(EngineError::SeriesDecided);
        assert_eq!(check_next(4, &confirmed(&[A, A, A]), None), decided);
        assert_eq!(check_next(6, &confirmed(&[B, B, B, B]), None), decided);
        // Pelos enviados, ainda sem confirmar, e o que vem depois não vale.
        let pending = [g(A, Pending), g(A, Pending), g(A, Pending)];
        assert_eq!(check_next(4, &pending, None), decided);
        let beyond = evaluate(
            4,
            &[
                g(A, Confirmed),
                g(A, Confirmed),
                g(A, Confirmed),
                g(B, Pending),
            ],
        );
        assert_eq!((beyond.standing, beyond.decided), (3, decision(A, 3, 0)));
        // Sem maioria, com todos os mapas: acabou.
        for (n, w) in [(4, &[A, B, A, B][..]), (6, &[A, B, A, B, A, B][..])] {
            assert_eq!(
                check_next(n, &confirmed(w), None),
                Err(EngineError::NoGamesLeft { best_of: n })
            );
        }
        // Antes disso, o próximo pode vir.
        assert_eq!(check_next(4, &confirmed(&[A, A]), None), Ok(()));
        assert_eq!(check_next(6, &confirmed(&[A, A, A, B, B]), None), Ok(()));
    }

    #[test]
    fn games_left_in_an_even_series_stops_at_the_maps_not_sent() {
        assert_eq!(games_left(4, &[]), 3);
        assert_eq!(games_left(4, &confirmed(&[A, A])), 1);
        assert_eq!(games_left(4, &confirmed(&[A, B])), 2);
        assert_eq!(games_left(4, &confirmed(&[A, B, A])), 1);
        assert_eq!(games_left(4, &confirmed(&[A, B, A, B])), 0);
        assert_eq!(games_left(4, &confirmed(&[A, A, A])), 0);
        assert_eq!(games_left(6, &[]), 4);
        assert_eq!(games_left(6, &confirmed(&[A, B, A, B])), 2);
        assert_eq!(games_left(6, &confirmed(&[A, B, A, B, A, B])), 0);
    }

    #[test]
    fn correcting_a_best_of_four_moves_between_win_and_draw() {
        let won = [on(1, A), on(2, B), on(3, A), on(4, A)];
        let draw = put(4, &won, 3, on(4, B)).unwrap();
        assert_eq!((draw.standing, draw.decided), (4, drawn(2)));
        let tied = [on(1, A), on(2, B), on(3, A), on(4, B)];
        let win = put(4, &tied, 3, on(4, A)).unwrap();
        assert_eq!((win.standing, win.decided), (4, decision(A, 3, 1)));
        // Corrigir para 3 x 0 no terceiro mapa deixa o quarto de fora.
        let swept = put(4, &tied, 1, on(2, A)).unwrap();
        assert_eq!((swept.standing, swept.decided), (3, decision(A, 3, 0)));
    }

    #[test]
    fn odd_series_never_draw() {
        // Nenhum placar sem maioria fecha a série ímpar.
        assert_eq!(
            evaluate(3, &[g(A, Confirmed), g(B, Confirmed)]).decided,
            None
        );
        let four = [
            g(A, Confirmed),
            g(B, Confirmed),
            g(A, Confirmed),
            g(B, Confirmed),
        ];
        assert_eq!(evaluate(5, &four).decided, None);
    }

    #[test]
    fn the_winner_is_who_scored_more() {
        assert_eq!(winner_from_frags(1, 20, 10), Ok(A));
        assert_eq!(winner_from_frags(2, 3, 30), Ok(B));
        assert_eq!(
            winner_from_frags(2, 10, 10),
            Err(EngineError::GameTied { game: 2 })
        );
    }

    #[test]
    fn negative_frags_follow_the_same_rule() {
        // Suicídio e queda tiram frag: quem fez mais venceu, mesmo abaixo de 0.
        assert_eq!(winner_from_frags(1, 20, -1), Ok(A));
        assert_eq!(winner_from_frags(1, -1, -3), Ok(A));
        assert_eq!(winner_from_frags(1, -3, -1), Ok(B));
        assert_eq!(winner_from_frags(1, 0, -1), Ok(A));
        assert_eq!(winner_from_frags(1, -999, 999), Ok(B));
        assert_eq!(
            winner_from_frags(3, -2, -2),
            Err(EngineError::GameTied { game: 3 })
        );
    }
}
