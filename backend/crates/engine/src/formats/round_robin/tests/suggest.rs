//! Testes de `suggest`: a série que cada time joga agora, no `schedule` real.

use std::collections::BTreeMap;

use super::*;
use proptest::prelude::*;

use SeriesProgress::{Called, Open, Playing, Settling};

const SHORT: SeriesProgress = Playing { long: false };
const LONG: SeriesProgress = Playing { long: true };

/// A partida entre `x` e `y`.
fn between(b: &RoundRobinBracket, x: i64, y: i64) -> Match {
    let (x, y) = (x.min(y), x.max(y));
    *b.matches()
        .iter()
        .find(|m| {
            matches!(m.state, MatchState::Ready { a, b: other } if (a.0, other.0) == (x, y))
                || matches!(m.state, MatchState::Completed { a, b: other, .. } if (a.0, other.0) == (x, y))
        })
        .unwrap_or_else(|| panic!("sem partida {x} x {y}"))
}

/// Decide as séries dadas (vence o seed melhor).
fn decide(b: &mut RoundRobinBracket, pairs: &[(i64, i64)]) {
    for &(x, y) in pairs {
        let m = between(b, x, y);
        b.report(played(&m, p(x.min(y)), 0)).unwrap();
    }
}

/// Decide todas as séries, menos as dadas.
fn decide_except(b: &mut RoundRobinBracket, keep: &[(i64, i64)]) {
    let all: Vec<(i64, i64)> = b
        .matches()
        .iter()
        .filter_map(|m| match m.state {
            MatchState::Ready { a, b } => Some((a.0, b.0)),
            _ => None,
        })
        .filter(|pair| !keep.contains(pair))
        .collect();
    decide(b, &all);
}

/// O que `suggest` devolve, como time -> (x, y) da série. O progresso de
/// cada série pronta é o das marcas; as outras estão `Open`.
fn run(b: &RoundRobinBracket, marks: &[((i64, i64), SeriesProgress)]) -> BTreeMap<i64, (i64, i64)> {
    let by_id: HashMap<MatchId, SeriesProgress> = marks
        .iter()
        .map(|&((x, y), progress)| (between(b, x, y).id, progress))
        .collect();
    let out = suggest(b.matches(), |m| by_id.get(&m.id).copied().unwrap_or(Open));
    out.into_iter()
        .map(|(team, id)| {
            let MatchState::Ready { a, b: other } = b.get(id).unwrap().state else {
                panic!("sugestão que não está pronta: {id:?}");
            };
            (team.0, (a.0, other.0))
        })
        .collect()
}

fn expect(got: BTreeMap<i64, (i64, i64)>, want: &[(i64, (i64, i64))]) {
    assert_eq!(got, want.iter().copied().collect::<BTreeMap<_, _>>());
}

fn bracket(n: i64) -> RoundRobinBracket {
    RoundRobinBracket::new(config(), seeds(n)).unwrap()
}

#[test]
fn the_start_follows_round_one() {
    expect(
        run(&bracket(4), &[]),
        &[(1, (1, 4)), (4, (1, 4)), (2, (2, 3)), (3, (2, 3))],
    );
    expect(
        run(&bracket(6), &[]),
        &[
            (1, (1, 6)),
            (6, (1, 6)),
            (2, (2, 5)),
            (5, (2, 5)),
            (3, (3, 4)),
            (4, (3, 4)),
        ],
    );
}

#[test]
fn a_decided_round_moves_everyone_to_the_next() {
    let mut four = bracket(4);
    decide(&mut four, &[(1, 4), (2, 3)]);
    expect(
        run(&four, &[]),
        &[(1, (1, 3)), (3, (1, 3)), (2, (2, 4)), (4, (2, 4))],
    );
    let mut six = bracket(6);
    decide(&mut six, &[(1, 6), (2, 5), (3, 4)]);
    expect(
        run(&six, &[]),
        &[
            (1, (1, 3)),
            (3, (1, 3)),
            (2, (2, 6)),
            (6, (2, 6)),
            (4, (4, 5)),
            (5, (4, 5)),
        ],
    );
}

#[test]
fn odd_count_never_suggests_the_rest() {
    let b = bracket(5);
    let out = run(&b, &[]);
    assert_eq!(out.len(), 5);
    // Quem folga na rodada 1 (seed 1) recebe uma série de outra rodada.
    assert_ne!(out[&1], (2, 5));
    assert_eq!(out[&2], (2, 5));
    assert_eq!(out[&3], (3, 4));
}

#[test]
fn a_team_in_play_keeps_its_series_even_from_a_later_round() {
    // 1 x 3 é da rodada 2; 1 ainda tem 1 x 4 aberta, da rodada 1.
    let b = bracket(4);
    expect(
        run(&b, &[((1, 3), SHORT)]),
        &[(1, (1, 3)), (3, (1, 3)), (2, (2, 3)), (4, (1, 4))],
    );
}

#[test]
fn a_short_opponent_is_worth_waiting_for() {
    // 3 e 4 jogam a última rodada; 1 e 2 estão livres, e os adversários da
    // vez (3 e 4) acabam logo: ninguém salta para 1 x 2.
    let mut b = bracket(4);
    decide(&mut b, &[(1, 4), (2, 3)]);
    expect(
        run(&b, &[((3, 4), SHORT)]),
        &[(1, (1, 3)), (3, (3, 4)), (2, (2, 4)), (4, (3, 4))],
    );
    // Sem as decididas, é igual: 2 fica com a dele em ordem.
    let b = bracket(4);
    let got = run(&b, &[((1, 3), SHORT)]);
    assert_eq!((got[&2], got[&4]), ((2, 3), (1, 4)));
}

#[test]
fn a_long_opponent_makes_two_free_teams_jump_to_their_series() {
    let mut b = bracket(4);
    decide(&mut b, &[(1, 4), (2, 3)]);
    expect(
        run(&b, &[((3, 4), LONG)]),
        &[(1, (1, 2)), (2, (1, 2)), (3, (3, 4)), (4, (3, 4))],
    );
    // No início, 1 x 3 longa deixa 2 e 4 livres: saltam para 2 x 4.
    expect(
        run(&bracket(4), &[((1, 3), LONG)]),
        &[(1, (1, 3)), (3, (1, 3)), (2, (2, 4)), (4, (2, 4))],
    );
}

#[test]
fn no_jump_when_a_team_has_its_own_series_in_order() {
    // 3 x 4 longa; 2 x 5 aberta e em ordem para os dois (passo 3). Sobram 1
    // (adversário da vez ocupado) e 6, mas não há série aberta entre eles:
    // ninguém salta, cada um fica com a sua em ordem (passo 5).
    let mut b = bracket(6);
    decide(&mut b, &[(1, 6)]);
    expect(
        run(&b, &[((3, 4), LONG)]),
        &[
            (3, (3, 4)),
            (4, (3, 4)),
            (2, (2, 5)),
            (5, (2, 5)),
            (1, (1, 3)),
            (6, (2, 6)),
        ],
    );
}

#[test]
fn six_players_jump_to_the_open_series_between_two_stuck_teams() {
    // 1 e 5 esperam 3 e 4 (longa); 2 x 6 é a vez de 2 e 6; 1 x 5 é aberta.
    let mut b = bracket(6);
    decide(&mut b, &[(1, 6), (2, 5)]);
    expect(
        run(&b, &[((3, 4), LONG)]),
        &[
            (3, (3, 4)),
            (4, (3, 4)),
            (2, (2, 6)),
            (6, (2, 6)),
            (1, (1, 5)),
            (5, (1, 5)),
        ],
    );
}

#[test]
fn settling_counts_as_free() {
    let b = bracket(4);
    // 1 e 4 aguardam a confirmação de 1 x 4 e já podem jogar outra.
    expect(
        run(&b, &[((1, 4), Settling)]),
        &[(1, (1, 3)), (4, (2, 4)), (2, (2, 3)), (3, (2, 3))],
    );
    // Sem série aberta, fica a que espera.
    let mut b = bracket(4);
    decide_except(&mut b, &[(1, 4)]);
    expect(run(&b, &[((1, 4), Settling)]), &[(1, (1, 4)), (4, (1, 4))]);
}

#[test]
fn a_call_takes_both_sides_unless_one_is_playing() {
    let b = bracket(4);
    expect(
        run(&b, &[((1, 4), Called)]),
        &[(1, (1, 4)), (4, (1, 4)), (2, (2, 3)), (3, (2, 3))],
    );
    // 1 está jogando 1 x 3: fica nela, e 4 vai para a chamada.
    expect(
        run(&b, &[((1, 4), Called), ((1, 3), SHORT)]),
        &[(1, (1, 3)), (3, (1, 3)), (4, (1, 4)), (2, (2, 3))],
    );
}

#[test]
fn only_ready_matches_reach_the_callback() {
    let mut b = bracket(4);
    decide(&mut b, &[(1, 4)]);
    let out = suggest(b.matches(), |m| {
        assert!(matches!(m.state, MatchState::Ready { .. }));
        Open
    });
    assert_eq!(out.len(), 4);
    let mut done = bracket(4);
    decide_except(&mut done, &[]);
    assert!(suggest(done.matches(), |_| panic!("sem partida pronta")).is_empty());
}

/// 0 aberta, 1 chamada, 2 em jogo curta, 3 em jogo longa, 4 fechando,
/// 5 decidida.
fn code(c: u8) -> SeriesProgress {
    match c {
        0 | 5 => Open,
        1 => Called,
        2 => SHORT,
        3 => LONG,
        _ => Settling,
    }
}

proptest! {
    #[test]
    fn every_team_with_a_ready_series_gets_one_of_its_own(
        n in prop::sample::select(vec![4i64, 6, 7, 8]),
        codes in proptest::collection::vec(0u8..6, 1..40),
    ) {
        let mut b = bracket(n);
        let ready: Vec<Match> = b.matches().iter().filter(|m| matches!(m.state, MatchState::Ready { .. })).copied().collect();
        for (i, m) in ready.iter().enumerate() {
            if codes[i % codes.len()] == 5 {
                let MatchState::Ready { a, b: other } = m.state else { unreachable!() };
                b.report(played(m, a.min(other), 0)).unwrap();
            }
        }
        let progress_of: HashMap<MatchId, SeriesProgress> = ready
            .iter()
            .enumerate()
            .map(|(i, m)| (m.id, code(codes[i % codes.len()])))
            .collect();
        let out = suggest(b.matches(), |m| progress_of[&m.id]);

        let mut with_ready: HashSet<ParticipantId> = HashSet::new();
        for m in b.matches() {
            if let MatchState::Ready { a, b: other } = m.state {
                with_ready.insert(a);
                with_ready.insert(other);
            }
        }
        prop_assert_eq!(out.keys().copied().collect::<HashSet<_>>(), with_ready);

        for (team, id) in &out {
            let m = b.get(*id).unwrap();
            let MatchState::Ready { a, b: other } = m.state else {
                return Err(TestCaseError::fail(format!("{id:?} não está pronta")));
            };
            prop_assert!(*team == a || *team == other);
            // Quem está em jogo fica na série em jogo; senão, na chamada.
            let mine: Vec<SeriesProgress> = b.matches().iter()
                .filter(|x| matches!(x.state, MatchState::Ready { a, b } if a == *team || b == *team))
                .map(|x| progress_of[&x.id])
                .collect();
            if mine.iter().any(|q| matches!(q, Playing { .. })) {
                let playing = matches!(progress_of[id], Playing { .. });
                prop_assert!(playing);
            } else if mine.contains(&Called) {
                prop_assert_eq!(progress_of[id], Called);
            }
            // Dois times que apontam um para o outro apontam para a mesma série.
            let opponent = if *team == a { other } else { a };
            let points_back = out.get(&opponent).is_some_and(|back| {
                matches!(b.get(*back).unwrap().state, MatchState::Ready { a, b } if a == *team || b == *team)
            });
            if points_back {
                prop_assert_eq!(out[&opponent], *id);
            }
        }
    }
}
