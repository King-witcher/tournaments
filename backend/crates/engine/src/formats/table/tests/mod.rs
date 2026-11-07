//! Testes da classificação: saldo de séries, pontos, estatísticas, empates e
//! cada desempate, na ordem padrão. Outras ordens: `tiebreakers.rs`; o menor
//! rating: `lower_rating.rs`.

mod lower_rating;
mod tiebreakers;
mod ties;

use super::*;
use crate::core::{BracketSide, EngineError, MatchId, MatchReport, SeriesScore, Slot, StageReport};
use Criterion::*;

/// As regras de cada formato com os desempates padrão.
const ROUND_ROBIN: Rules = Rules::round_robin(Tiebreakers::ROUND_ROBIN);
const SWISS: Rules = Rules::swiss(Tiebreakers::SWISS);

fn p(n: i64) -> ParticipantId {
    ParticipantId(n)
}

fn mid(index: u16) -> MatchId {
    MatchId {
        side: BracketSide::RoundRobin,
        round: 1,
        index,
    }
}

/// Uma partida por par, melhor de `best_of`, na ordem (a partida `i` é
/// `mid(i + 1)`), e um bye para cada um de `rests`, depois.
fn league_of(best_of: u8, rules: Rules, n: i64, pairs: &[(i64, i64)], rests: &[i64]) -> League {
    let mut matches: Vec<Match> = pairs
        .iter()
        .enumerate()
        .map(|(i, &(a, b))| pairing(mid(i as u16 + 1), p(a), p(b), best_of))
        .collect();
    for (k, &r) in rests.iter().enumerate() {
        matches.push(rest(mid((pairs.len() + k) as u16 + 1), p(r), best_of));
    }
    League::new(rules, (1..=n).map(p).collect(), matches)
}

/// A liga em md3.
fn league(rules: Rules, n: i64, pairs: &[(i64, i64)], rests: &[i64]) -> League {
    league_of(3, rules, n, pairs, rests)
}

/// A partida `index` (a partir de 1) vencida por `winner` com `loser_maps`
/// para o outro, numa md3, com os frags (do vencedor, do perdedor).
fn won(l: &mut League, index: u16, winner: i64, loser_maps: u8, frags: (i32, i32)) {
    report(l, index, winner, loser_maps, Some(frags));
}

/// Como `won`, com algum mapa sem frags.
fn won_without_frags(l: &mut League, index: u16, winner: i64, loser_maps: u8) {
    report(l, index, winner, loser_maps, None);
}

fn report(l: &mut League, index: u16, winner: i64, loser_maps: u8, frags: Option<(i32, i32)>) {
    let m = *l.get(mid(index)).unwrap();
    let a_won = m.a == Slot::Participant(p(winner));
    let needed = wins_needed(m.best_of);
    l.report(StageReport {
        report: MatchReport {
            match_id: mid(index),
            outcome: Outcome::Played {
                winner: p(winner),
                score: SeriesScore {
                    winner: needed,
                    loser: loser_maps,
                },
            },
        },
        frags: frags.map(|(w, x)| {
            if a_won {
                SeriesFrags { a: w, b: x }
            } else {
                SeriesFrags { a: x, b: w }
            }
        }),
    })
    .unwrap();
}

/// A partida `index` empatada, com os frags dos lados `a` e `b`.
fn drew(l: &mut League, index: u16, frags: Option<(i32, i32)>) {
    let maps = l.get(mid(index)).unwrap().best_of / 2;
    l.report(StageReport {
        report: MatchReport {
            match_id: mid(index),
            outcome: Outcome::Drawn { maps },
        },
        frags: frags.map(|(a, b)| SeriesFrags { a, b }),
    })
    .unwrap();
}

fn order(l: &League) -> Vec<i64> {
    l.table().iter().map(|r| r.participant.0).collect()
}

fn ahead(l: &League) -> Vec<Option<Criterion>> {
    l.table().iter().map(|r| r.ahead_by).collect()
}

fn row(l: &League, n: i64) -> TableRow {
    *l.table().iter().find(|r| r.participant == p(n)).unwrap()
}

#[test]
fn nothing_played_is_the_seed_order() {
    let l = league(ROUND_ROBIN, 4, &[(1, 2), (3, 4)], &[]);
    assert_eq!(order(&l), [1, 2, 3, 4]);
    assert_eq!(ahead(&l), [Some(Seed), Some(Seed), Some(Seed), None]);
    let positions: Vec<u16> = l.table().iter().map(|r| r.position).collect();
    assert_eq!(positions, [1, 2, 3, 4]);
}

#[test]
fn stats_add_up() {
    let mut l = league(ROUND_ROBIN, 3, &[(1, 2), (1, 3), (2, 3)], &[]);
    won(&mut l, 1, 2, 1, (30, 25));
    won(&mut l, 2, 1, 0, (20, 5));
    let r = row(&l, 1);
    assert_eq!((r.points, r.played, r.wins, r.losses), (1, 2, 1, 1));
    assert_eq!((r.series_diff, r.draws), (0, 0));
    assert_eq!((r.maps_won, r.maps_lost, r.map_diff), (3, 2, 1));
    assert_eq!((r.frags_for, r.frags_against, r.frag_diff), (45, 35, 10));
    assert_eq!((r.byes, r.buchholz), (0, None));
    let r = row(&l, 3);
    assert_eq!((r.points, r.played, r.losses, r.map_diff), (0, 1, 1, -2));
    assert_eq!(r.series_diff, -1);
}

#[test]
fn series_diff_comes_first_so_more_games_do_not_put_you_ahead() {
    // No meio do torneio: 2 venceu duas e perdeu uma, 3 venceu a única que
    // jogou. Por pontos, 2 estaria na frente só por ter jogado mais.
    let mut l = league(ROUND_ROBIN, 4, &[(2, 4), (2, 1), (2, 3)], &[]);
    won(&mut l, 1, 2, 0, (0, 0));
    won(&mut l, 2, 2, 0, (0, 0));
    won(&mut l, 3, 3, 0, (0, 0));
    assert_eq!((row(&l, 2).points, row(&l, 2).series_diff), (2, 1));
    assert_eq!((row(&l, 3).points, row(&l, 3).series_diff), (1, 1));
    // Mesmo saldo: quem venceu o confronto direto fica à frente.
    assert_eq!(order(&l), [3, 2, 1, 4]);
    assert_eq!(ahead(&l)[0], Some(HeadToHead));
    assert_eq!(ahead(&l)[1], Some(SeriesDiff));
}

#[test]
fn head_to_head_beats_a_better_map_diff() {
    // 2 e 3 com saldo +1; 2 tem saldo de mapas melhor, mas 3 venceu 2.
    let pairs = [(2, 3), (2, 4), (2, 5), (1, 3), (3, 4)];
    let mut l = league(ROUND_ROBIN, 5, &pairs, &[]);
    won(&mut l, 1, 3, 1, (0, 0));
    won(&mut l, 2, 2, 0, (0, 0));
    won(&mut l, 3, 2, 0, (0, 0));
    won(&mut l, 4, 3, 1, (0, 0));
    won(&mut l, 5, 4, 1, (0, 0));
    assert_eq!((row(&l, 2).series_diff, row(&l, 3).series_diff), (1, 1));
    assert!(row(&l, 2).map_diff > row(&l, 3).map_diff);
    assert_eq!(order(&l), [3, 2, 4, 1, 5]);
    assert_eq!(
        ahead(&l),
        [
            Some(HeadToHead),
            Some(SeriesDiff),
            Some(SeriesDiff),
            Some(MapDiff),
            None
        ]
    );
}

#[test]
fn three_way_cycle_falls_to_map_diff() {
    // 1 > 2 > 3 > 1: a mini-tabela empata os três.
    let mut l = league(ROUND_ROBIN, 3, &[(1, 2), (2, 3), (1, 3)], &[]);
    won(&mut l, 1, 1, 0, (0, 0));
    won(&mut l, 2, 2, 1, (0, 0));
    won(&mut l, 3, 3, 1, (0, 0));
    // Saldos: 1 = +2 − 1, 2 = −2 + 1, 3 = +1 − 1.
    assert_eq!(order(&l), [1, 3, 2]);
    assert_eq!(ahead(&l), [Some(MapDiff), Some(MapDiff), None]);
}

#[test]
fn mini_table_is_redone_among_who_stays_tied() {
    // 1, 2, 3 e 4 com saldo 0, e todos se enfrentaram. Na mini-tabela, 1 e 4
    // têm +1, 2 e 3 têm −1; refeita entre 1 e 4, 1 venceu 4, e entre 2 e 3, 2
    // venceu 3. Pelo saldo de mapas, 4 passaria 1 e 3 passaria 2.
    let pairs = [
        (1, 4),
        (1, 2),
        (2, 4),
        (3, 4),
        (2, 3),
        (1, 3),
        (2, 5),
        (3, 5),
        (1, 6),
        (4, 6),
    ];
    let mut l = league(ROUND_ROBIN, 6, &pairs, &[]);
    won(&mut l, 1, 1, 1, (0, 0));
    won(&mut l, 2, 1, 1, (0, 0));
    won(&mut l, 3, 4, 0, (0, 0));
    won(&mut l, 4, 4, 0, (0, 0));
    won(&mut l, 5, 2, 1, (0, 0));
    won(&mut l, 6, 3, 0, (0, 0));
    won(&mut l, 7, 2, 0, (0, 0));
    won(&mut l, 8, 3, 0, (0, 0));
    won(&mut l, 9, 6, 0, (0, 0));
    won(&mut l, 10, 6, 0, (0, 0));
    assert!(row(&l, 4).map_diff > row(&l, 1).map_diff);
    assert!(row(&l, 3).map_diff > row(&l, 2).map_diff);
    assert_eq!(order(&l), [6, 1, 4, 2, 3, 5]);
    assert_eq!(
        ahead(&l),
        [
            Some(SeriesDiff),
            Some(HeadToHead),
            Some(HeadToHead),
            Some(HeadToHead),
            Some(SeriesDiff),
            None
        ]
    );
}

#[test]
fn head_to_head_returns_after_map_diff() {
    // Ciclo 3 > 2 > 1 > 3 (2 x 0), e 1, 2 e 3 vencem o 4 (1 por 2 x 0, 2 e 3
    // por 2 x 1). Saldos de mapas: 1 = +2, 2 = 3 = +1. Os frags favorecem o
    // 2 (+19 contra 0), mas 3 venceu a série entre os dois.
    let pairs = [(3, 2), (2, 1), (1, 3), (1, 4), (2, 4), (3, 4)];
    let mut l = league(ROUND_ROBIN, 4, &pairs, &[]);
    won(&mut l, 1, 3, 0, (10, 9));
    won(&mut l, 2, 2, 0, (30, 10));
    won(&mut l, 3, 1, 0, (10, 9));
    won(&mut l, 4, 1, 0, (0, 0));
    won(&mut l, 5, 2, 1, (0, 0));
    won(&mut l, 6, 3, 1, (0, 0));
    assert_eq!(
        (
            row(&l, 1).map_diff,
            row(&l, 2).map_diff,
            row(&l, 3).map_diff
        ),
        (2, 1, 1)
    );
    assert!(row(&l, 2).frag_diff > row(&l, 3).frag_diff);
    assert_eq!(order(&l), [1, 3, 2, 4]);
    assert_eq!(
        ahead(&l),
        [Some(MapDiff), Some(HeadToHead), Some(SeriesDiff), None]
    );
}

#[test]
fn head_to_head_returns_after_frag_diff() {
    // O mesmo ciclo, todas as séries 2 x 0: os saldos de mapas empatam. O 1
    // tem mais frags; 2 e 3 empatam em frags (0 e 0) e o seed poria o 2 na
    // frente, mas 3 venceu a série entre os dois.
    let pairs = [(3, 2), (2, 1), (1, 3), (1, 4), (2, 4), (3, 4)];
    let mut l = league(ROUND_ROBIN, 4, &pairs, &[]);
    won(&mut l, 1, 3, 0, (10, 9));
    won(&mut l, 2, 2, 0, (10, 9));
    won(&mut l, 3, 1, 0, (10, 9));
    won(&mut l, 4, 1, 0, (50, 0));
    won(&mut l, 5, 2, 0, (0, 0));
    won(&mut l, 6, 3, 0, (0, 0));
    assert_eq!(
        (
            row(&l, 1).map_diff,
            row(&l, 2).map_diff,
            row(&l, 3).map_diff
        ),
        (2, 2, 2)
    );
    assert_eq!(
        (
            row(&l, 1).frag_diff,
            row(&l, 2).frag_diff,
            row(&l, 3).frag_diff
        ),
        (50, 0, 0)
    );
    assert_eq!(order(&l), [1, 3, 2, 4]);
    assert_eq!(
        ahead(&l),
        [Some(FragDiff), Some(HeadToHead), Some(SeriesDiff), None]
    );
}

#[test]
fn head_to_head_returns_for_a_pair_in_a_partial_table() {
    // 1, 2 e 3 com saldo +1; 3 não enfrentou os outros, então o confronto
    // direto não vale para o trio. O saldo de mapas põe o 3 na frente (+2
    // contra +1 e +1); 1 e 2 voltam ao confronto direto, que 2 venceu, ainda
    // que os frags e o seed favoreçam o 1.
    let pairs = [(1, 2), (3, 4), (1, 5), (1, 6)];
    let mut l = league(ROUND_ROBIN, 6, &pairs, &[]);
    won(&mut l, 1, 2, 1, (30, 31));
    won(&mut l, 2, 3, 0, (0, 0));
    won(&mut l, 3, 1, 1, (10, 0));
    won(&mut l, 4, 1, 1, (10, 0));
    assert!(row(&l, 1).frag_diff > row(&l, 2).frag_diff);
    assert_eq!(order(&l)[..3], [3, 2, 1]);
    assert_eq!(ahead(&l)[0], Some(MapDiff));
    assert_eq!(ahead(&l)[1], Some(HeadToHead));
}

#[test]
fn head_to_head_needs_everyone_to_have_met() {
    // 1, 2 e 3 com saldo +1; 1 venceu 2, mas 3 não enfrentou nenhum dos dois
    // (no suíço, o comum): sem confronto direto, o saldo de mapas, que põe o
    // 2 à frente do 1 mesmo tendo perdido para ele.
    let pairs = [(1, 2), (3, 4), (2, 5), (2, 6)];
    let mut l = league(ROUND_ROBIN, 6, &pairs, &[]);
    won(&mut l, 1, 1, 1, (0, 0));
    won(&mut l, 2, 3, 0, (0, 0));
    won(&mut l, 3, 2, 0, (0, 0));
    won(&mut l, 4, 2, 0, (0, 0));
    assert_eq!(order(&l)[..3], [2, 3, 1]);
    assert_eq!(ahead(&l)[0], Some(MapDiff));
}

#[test]
fn frags_then_seed() {
    // 2 e 3 vencem 2 x 1 quem não joga entre eles; 3 com mais frags.
    let mut l = league(ROUND_ROBIN, 5, &[(2, 4), (3, 5), (1, 4)], &[]);
    won(&mut l, 1, 2, 1, (40, 39));
    won(&mut l, 2, 3, 1, (50, 30));
    won(&mut l, 3, 1, 1, (40, 39));
    // 1 e 2 empatam em tudo (saldo de séries, de mapas, de frags): o seed. 4
    // perdeu duas por 2 x 1, 5 uma.
    assert_eq!(order(&l), [3, 1, 2, 5, 4]);
    assert_eq!(
        ahead(&l),
        [
            Some(FragDiff),
            Some(Seed),
            Some(SeriesDiff),
            Some(SeriesDiff),
            None
        ]
    );
}

#[test]
fn frags_only_compare_who_has_them_all() {
    // 1 e 2 empatados em tudo antes dos frags; 2 tem +40, mas a série do 1
    // veio sem frags: vale o seed.
    let mut l = league(ROUND_ROBIN, 4, &[(1, 3), (2, 4)], &[]);
    won_without_frags(&mut l, 1, 1, 1);
    won(&mut l, 2, 2, 1, (50, 10));
    assert_eq!(order(&l), [1, 2, 3, 4]);
    assert_eq!(ahead(&l)[0], Some(Seed));
    assert!(!row(&l, 1).frags_complete);
    assert!(row(&l, 2).frags_complete);
    assert_eq!(row(&l, 2).frag_diff, 40);
}

#[test]
fn negative_frags_count_with_their_sign() {
    // Suicídio e queda tiram frag. 1, 2 e 3 vencem 2 x 1 quem não joga entre
    // eles: 2 por -1 x -3 (+2), 3 por 20 x -1 (+21), 1 por 0 x -1 (+1).
    let mut l = league(ROUND_ROBIN, 5, &[(2, 4), (3, 5), (1, 4)], &[]);
    won(&mut l, 1, 2, 1, (-1, -3));
    won(&mut l, 2, 3, 1, (20, -1));
    won(&mut l, 3, 1, 1, (0, -1));
    assert_eq!(order(&l), [3, 2, 1, 5, 4]);
    assert_eq!(
        ahead(&l),
        [
            Some(FragDiff),
            Some(FragDiff),
            Some(SeriesDiff),
            Some(SeriesDiff),
            None
        ]
    );
    // Frags pró negativos com saldo positivo, e o saldo de quem perdeu as
    // duas somando os dois lados com sinal.
    let r = row(&l, 2);
    assert_eq!((r.frags_for, r.frags_against, r.frag_diff), (-1, -3, 2));
    let r = row(&l, 4);
    assert_eq!((r.frags_for, r.frags_against, r.frag_diff), (-4, -1, -3));
    let r = row(&l, 5);
    assert_eq!((r.frags_for, r.frags_against, r.frag_diff), (-1, 20, -21));
}

#[test]
fn walkover_is_a_clean_win_without_frags() {
    let mut l = league(ROUND_ROBIN, 2, &[(1, 2)], &[]);
    l.report(StageReport {
        report: MatchReport {
            match_id: mid(1),
            outcome: Outcome::Walkover { winner: p(2) },
        },
        frags: Some(SeriesFrags { a: 3, b: 10 }),
    })
    .unwrap();
    let r = row(&l, 2);
    assert_eq!((r.points, r.wins, r.played), (1, 1, 1));
    // Vale 2 x 0 na md3; os frags do W.O. não contam, nem como faltando.
    assert_eq!((r.maps_won, r.maps_lost, r.map_diff), (2, 0, 2));
    assert_eq!(
        (r.frags_for, r.frags_against, r.frags_complete),
        (0, 0, true)
    );
    assert_eq!(row(&l, 1).map_diff, -2);
    assert_eq!(order(&l), [2, 1]);
}

#[test]
fn round_robin_rest_is_worth_nothing() {
    let l = league(ROUND_ROBIN, 3, &[(1, 2)], &[3]);
    let r = row(&l, 3);
    assert_eq!((r.points, r.series_diff, r.byes, r.played), (0, 0, 0, 0));
}

#[test]
fn swiss_bye_is_a_point_and_buchholz_comes_next() {
    // Rodada 1: 1 > 2, 4 > 5, 2 > 3 (de outra rodada), bye do 6.
    let pairs = [(1, 2), (4, 5), (2, 3)];
    let mut l = league(SWISS, 6, &pairs, &[6]);
    won(&mut l, 1, 1, 0, (0, 0));
    won(&mut l, 2, 4, 0, (0, 0));
    won(&mut l, 3, 2, 0, (0, 0));
    let r = row(&l, 6);
    assert_eq!((r.points, r.byes, r.played, r.buchholz), (1, 1, 0, Some(0)));
    // Com um ponto: 1 e 2 têm Buchholz 1 (1 venceu 2 no confronto); 4 e 6,
    // Buchholz 0, separados pelo saldo de mapas (o bye não soma mapa). Sem
    // ponto: 3 e 5 com Buchholz 1, empatados até o seed.
    assert_eq!(order(&l), [1, 2, 4, 6, 3, 5]);
    assert_eq!(
        ahead(&l),
        [
            Some(HeadToHead),
            Some(Buchholz),
            Some(MapDiff),
            Some(Points),
            Some(Seed),
            None
        ]
    );
    let buchholz: Vec<Option<u32>> = l.table().iter().map(|r| r.buchholz).collect();
    assert_eq!(
        buchholz,
        [Some(1), Some(1), Some(0), Some(0), Some(1), Some(1)]
    );
}

#[test]
fn placements_follow_the_table_without_ties() {
    let mut l = league(SWISS, 3, &[(1, 2), (2, 3)], &[]);
    won(&mut l, 1, 2, 0, (0, 0));
    won(&mut l, 2, 3, 0, (0, 0));
    let placements: Vec<(i64, u16, u16)> = l
        .placements()
        .iter()
        .map(|x| (x.participant.0, x.rank, x.tied))
        .collect();
    assert_eq!(placements, [(3, 1, 1), (2, 2, 1), (1, 3, 1)]);
}

// ---- séries pares ----

fn md2(n: i64, pairs: &[(i64, i64)]) -> League {
    league_of(2, ROUND_ROBIN, n, pairs, &[])
}

#[test]
fn a_draw_is_worth_nothing_and_counts_one_map_each() {
    let mut l = md2(3, &[(1, 2), (1, 3), (2, 3)]);
    drew(&mut l, 1, Some((30, 25)));
    // Os frags somam para cada lado, sem vencedor nem perdedor.
    let r = row(&l, 1);
    assert_eq!((r.played, r.wins, r.draws, r.losses), (1, 0, 1, 0));
    assert_eq!((r.points, r.series_diff), (0, 0));
    assert_eq!((r.maps_won, r.maps_lost, r.map_diff), (1, 1, 0));
    assert_eq!((r.frags_for, r.frags_against, r.frag_diff), (30, 25, 5));
    let r = row(&l, 2);
    assert_eq!((r.played, r.draws), (1, 1));
    assert_eq!((r.frags_for, r.frags_against, r.frag_diff), (25, 30, -5));
    // Quem ganhou uma série passa quem empatou; quem perdeu fica atrás.
    report(&mut l, 2, 3, 0, Some((40, 10)));
    assert_eq!(row(&l, 3).series_diff, 1);
    assert_eq!(row(&l, 1).series_diff, -1);
    assert_eq!(order(&l), [3, 2, 1]);
    assert_eq!(ahead(&l), [Some(SeriesDiff), Some(SeriesDiff), None]);
    assert_eq!((row(&l, 3).maps_won, row(&l, 3).maps_lost), (2, 0));
}

#[test]
fn a_draw_between_two_does_not_separate_them() {
    // 1 e 2 empataram entre si e venceram o 3 por 2 x 0: o confronto direto
    // conta como realizado, vale 0 para os dois, e os frags decidem.
    let mut l = md2(3, &[(1, 2), (1, 3), (2, 3)]);
    drew(&mut l, 1, Some((10, 10)));
    report(&mut l, 2, 1, 0, Some((10, 9)));
    report(&mut l, 3, 2, 0, Some((30, 9)));
    assert_eq!((row(&l, 1).map_diff, row(&l, 2).map_diff), (2, 2));
    assert_eq!(order(&l), [2, 1, 3]);
    assert_eq!(ahead(&l), [Some(FragDiff), Some(SeriesDiff), None]);
}

#[test]
fn a_draw_counts_as_having_met() {
    // 1, 2 e 3 com saldo +1, e todos se enfrentaram: 1 x 2 empate, 1 > 3, 3 > 2.
    // Se o empate não contasse, o confronto direto não valeria para o trio e os
    // frags poriam o 2 na frente.
    let pairs = [(1, 2), (1, 3), (2, 3), (2, 4), (2, 5), (3, 4)];
    let mut l = md2(5, &pairs);
    drew(&mut l, 1, Some((10, 10)));
    report(&mut l, 2, 1, 0, Some((10, 0)));
    report(&mut l, 3, 3, 0, Some((10, 0)));
    report(&mut l, 4, 2, 0, Some((50, 0)));
    report(&mut l, 5, 2, 0, Some((50, 0)));
    report(&mut l, 6, 3, 0, Some((10, 0)));
    let diffs = [1, 2, 3].map(|n| row(&l, n).series_diff);
    assert_eq!(diffs, [1, 1, 1]);
    assert!(row(&l, 2).frag_diff > row(&l, 1).frag_diff);
    assert_eq!(order(&l), [1, 3, 2, 5, 4]);
    assert_eq!(
        ahead(&l)[..3],
        [Some(HeadToHead), Some(HeadToHead), Some(SeriesDiff)]
    );
}

#[test]
fn the_mini_table_is_a_balance_in_the_round_robin() {
    // De 1 a 5, todos com saldo 0 e todos se enfrentaram. O 2 só empatou (0
    // vitórias, saldo 0); o 1 venceu uma e perdeu duas (1 vitória, saldo −1).
    // Pelas vitórias, o 1 ficaria à frente do 2. 6 e 7 só acertam o saldo.
    let pairs = [
        (1, 2),
        (2, 3),
        (2, 4),
        (2, 5),
        (1, 3),
        (1, 4),
        (1, 5),
        (3, 4),
        (3, 5),
        (4, 5),
        (1, 6),
        (3, 7),
    ];
    let mut l = md2(7, &pairs);
    for draw in [1, 2, 3, 4, 10] {
        drew(&mut l, draw, None);
    }
    report(&mut l, 5, 1, 0, None);
    report(&mut l, 6, 4, 0, None);
    report(&mut l, 7, 5, 0, None);
    report(&mut l, 8, 3, 0, None);
    report(&mut l, 9, 3, 0, None);
    report(&mut l, 11, 1, 0, None);
    report(&mut l, 12, 7, 0, None);
    let diffs = [1, 2, 3, 4, 5].map(|n| row(&l, n).series_diff);
    assert_eq!(diffs, [0, 0, 0, 0, 0]);
    assert_eq!(order(&l), [7, 3, 2, 4, 5, 1, 6]);
    assert_eq!(
        ahead(&l),
        [
            Some(SeriesDiff),
            Some(HeadToHead),
            Some(Seed),
            Some(Seed),
            Some(HeadToHead),
            Some(SeriesDiff),
            None
        ]
    );
}

#[test]
fn a_walkover_in_best_of_two_is_worth_two_nil() {
    let mut l = md2(2, &[(1, 2)]);
    l.report(StageReport::from(MatchReport {
        match_id: mid(1),
        outcome: Outcome::Walkover { winner: p(2) },
    }))
    .unwrap();
    let r = row(&l, 2);
    assert_eq!((r.wins, r.series_diff), (1, 1));
    assert_eq!((r.maps_won, r.maps_lost), (2, 0));
    assert_eq!(row(&l, 1).series_diff, -1);
}

#[test]
fn the_map_diff_is_twice_the_series_diff_in_best_of_two() {
    let pairs = [(1, 2), (1, 3), (2, 3), (1, 4), (3, 4), (2, 4)];
    let mut l = md2(4, &pairs);
    drew(&mut l, 1, None);
    report(&mut l, 2, 3, 0, None);
    report(&mut l, 3, 2, 0, None);
    drew(&mut l, 4, None);
    report(&mut l, 5, 3, 0, None);
    report(&mut l, 6, 4, 0, None);
    for r in l.table() {
        assert_eq!(r.map_diff, 2 * r.series_diff, "{:?}", r.participant);
    }
}

#[test]
fn in_longer_even_series_the_map_diff_really_breaks_the_tie() {
    // 1 e 2 vencem quem não joga entre eles, mesmo saldo de séries: na md4, 3 x 0
    // vale +3 e 3 x 1 vale +2.
    let mut l = league_of(4, ROUND_ROBIN, 4, &[(1, 3), (2, 4)], &[]);
    report(&mut l, 1, 1, 1, None);
    report(&mut l, 2, 2, 0, None);
    assert_eq!((row(&l, 1).series_diff, row(&l, 2).series_diff), (1, 1));
    assert_eq!((row(&l, 1).map_diff, row(&l, 2).map_diff), (2, 3));
    assert_eq!(order(&l)[..2], [2, 1]);
    assert_eq!(ahead(&l)[0], Some(MapDiff));
}

#[test]
fn a_draw_is_half_the_maps_and_a_walkover_the_majority() {
    for (best_of, half, majority) in [(2u8, 1u16, 2u16), (4, 2, 3), (6, 3, 4)] {
        let mut l = league_of(best_of, ROUND_ROBIN, 3, &[(1, 2), (1, 3)], &[]);
        drew(&mut l, 1, None);
        l.report(StageReport::from(MatchReport {
            match_id: mid(2),
            outcome: Outcome::Walkover { winner: p(3) },
        }))
        .unwrap();
        let r = row(&l, 2);
        assert_eq!((r.draws, r.maps_won, r.maps_lost), (1, half, half));
        let r = row(&l, 3);
        assert_eq!((r.wins, r.maps_won, r.maps_lost), (1, majority, 0));
        assert_eq!(r.series_diff, 1);
        let r = row(&l, 1);
        assert_eq!((r.draws, r.losses, r.series_diff), (1, 1, -1));
    }
}

#[test]
fn each_even_series_takes_only_its_own_scores() {
    let score = |best_of, winner, loser| {
        let mut l = league_of(best_of, ROUND_ROBIN, 2, &[(1, 2)], &[]);
        l.report(StageReport::from(MatchReport {
            match_id: mid(1),
            outcome: Outcome::Played {
                winner: p(1),
                score: SeriesScore { winner, loser },
            },
        }))
    };
    for (best_of, winner, loser) in [(4, 3, 0), (4, 3, 1), (6, 4, 0), (6, 4, 1), (6, 4, 2)] {
        assert_eq!(score(best_of, winner, loser), Ok(()), "md{best_of}");
    }
    for (best_of, winner, loser) in [(4, 3, 2), (4, 2, 0), (6, 4, 3), (6, 3, 0)] {
        assert!(score(best_of, winner, loser).is_err(), "md{best_of}");
    }
    // E o empate tem a metade dos mapas da série.
    let mut l = league_of(4, ROUND_ROBIN, 2, &[(1, 2)], &[]);
    let draw = |maps| {
        StageReport::from(MatchReport {
            match_id: mid(1),
            outcome: Outcome::Drawn { maps },
        })
    };
    assert!(l.report(draw(1)).is_err());
    l.report(draw(2)).unwrap();
}

#[test]
fn only_best_of_two_takes_a_draw() {
    let mut l = league(ROUND_ROBIN, 2, &[(1, 2)], &[]);
    let draw = StageReport::from(MatchReport {
        match_id: mid(1),
        outcome: Outcome::Drawn { maps: 1 },
    });
    assert_eq!(
        l.report(draw),
        Err(EngineError::InvalidScore {
            best_of: 3,
            score: SeriesScore {
                winner: 1,
                loser: 1
            }
        })
    );
    // E o placar da md2 é 2 x 0: 2 x 1 são três mapas.
    let mut l = md2(2, &[(1, 2)]);
    let three_maps = StageReport::from(MatchReport {
        match_id: mid(1),
        outcome: Outcome::Played {
            winner: p(1),
            score: SeriesScore {
                winner: 2,
                loser: 1,
            },
        },
    });
    assert!(matches!(
        l.report(three_maps),
        Err(EngineError::InvalidScore { best_of: 2, .. })
    ));
    assert!(!l.all_decided());
    l.report(draw).unwrap();
    assert!(l.all_decided());
    assert_eq!(
        l.report(draw),
        Err(EngineError::MatchAlreadyReported(mid(1)))
    );
}

// ---- a mini-tabela do suíço ----

/// A ordem só pelo confronto direto: os outros critérios empatam todos.
fn by_head_to_head(rules: Rules, meetings: &[tiebreak::Meeting]) -> Vec<usize> {
    let value = |c: Criterion, i: usize| if c == Seed { -(i as i64) } else { 0 };
    tiebreak::order(3, rules, value, |_, _| true, meetings)
        .into_iter()
        .map(|(i, _)| i)
        .collect()
}

#[test]
fn the_swiss_keeps_counting_wins_in_the_mini_table() {
    // Só com revanche as duas contas divergem: 0 venceu o 1 duas vezes, o 1
    // venceu o 2 e o 2 venceu o 0. Vitórias: 2, 1 e 1, e o 1 venceu o 2 no
    // empate. Saldo: +1, −1 e 0.
    let meeting = |a, b, winner| tiebreak::Meeting {
        a,
        b,
        winner: Some(winner),
    };
    let meetings = [
        meeting(0, 1, 0),
        meeting(0, 1, 0),
        meeting(1, 2, 1),
        meeting(2, 0, 2),
    ];
    assert_eq!(by_head_to_head(SWISS, &meetings), [0, 1, 2]);
    assert_eq!(by_head_to_head(ROUND_ROBIN, &meetings), [0, 2, 1]);
}
