//! Testes da classificação: pontos, estatísticas e cada desempate.

use super::*;
use crate::core::{BracketSide, MatchId, MatchReport, SeriesScore, StageReport};
use Criterion::*;

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

/// Uma partida por par, na ordem (a partida `i` é `mid(i + 1)`), e um bye
/// para cada um de `rests`, depois.
fn league(rules: Rules, n: i64, pairs: &[(i64, i64)], rests: &[i64]) -> League {
    let mut matches: Vec<Match> = pairs
        .iter()
        .enumerate()
        .map(|(i, &(a, b))| pairing(mid(i as u16 + 1), p(a), p(b), 3))
        .collect();
    for (k, &r) in rests.iter().enumerate() {
        matches.push(rest(mid((pairs.len() + k) as u16 + 1), p(r), 3));
    }
    League::new(rules, (1..=n).map(p).collect(), matches)
}

/// A partida `index` (a partir de 1) vencida por `winner` com `loser_maps`
/// para o outro, numa md3, com os frags (do vencedor, do perdedor).
fn won(l: &mut League, index: u16, winner: i64, loser_maps: u8, frags: (u32, u32)) {
    report(l, index, winner, loser_maps, Some(frags));
}

/// Como `won`, com algum mapa sem frags.
fn won_without_frags(l: &mut League, index: u16, winner: i64, loser_maps: u8) {
    report(l, index, winner, loser_maps, None);
}

fn report(l: &mut League, index: u16, winner: i64, loser_maps: u8, frags: Option<(u32, u32)>) {
    l.report(StageReport {
        report: MatchReport {
            match_id: mid(index),
            outcome: Outcome::Played {
                winner: p(winner),
                score: SeriesScore {
                    winner: 2,
                    loser: loser_maps,
                },
            },
        },
        frags: frags.map(|(winner, loser)| SeriesFrags { winner, loser }),
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
    assert_eq!((r.maps_won, r.maps_lost, r.map_diff), (3, 2, 1));
    assert_eq!((r.frags_for, r.frags_against, r.frag_diff), (45, 35, 10));
    assert_eq!((r.byes, r.buchholz), (0, None));
    let r = row(&l, 3);
    assert_eq!((r.points, r.played, r.losses, r.map_diff), (0, 1, 1, -2));
}

#[test]
fn head_to_head_beats_a_better_map_diff() {
    // 2 e 3 com dois pontos; 2 tem saldo de mapas melhor, mas 3 venceu 2.
    let pairs = [(2, 3), (2, 4), (2, 5), (1, 3)];
    let mut l = league(ROUND_ROBIN, 5, &pairs, &[]);
    won(&mut l, 1, 3, 1, (0, 0));
    won(&mut l, 2, 2, 0, (0, 0));
    won(&mut l, 3, 2, 0, (0, 0));
    won(&mut l, 4, 3, 1, (0, 0));
    assert!(row(&l, 2).map_diff > row(&l, 3).map_diff);
    assert_eq!(order(&l), [3, 2, 1, 4, 5]);
    assert_eq!(
        ahead(&l),
        [
            Some(HeadToHead),
            Some(Points),
            Some(MapDiff),
            Some(Seed),
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
    // 1, 2, 3 e 4 com dois pontos, e todos se enfrentaram. Na mini-tabela,
    // 1 e 4 têm dois, 2 e 3 um; refeita entre 1 e 4, 1 venceu 4, e entre 2
    // e 3, 2 venceu 3. Pelo saldo de mapas, 4 passaria 1 e 3 passaria 2.
    let pairs = [
        (1, 4),
        (1, 2),
        (2, 4),
        (3, 4),
        (2, 3),
        (1, 3),
        (2, 5),
        (3, 5),
    ];
    let mut l = league(ROUND_ROBIN, 5, &pairs, &[]);
    won(&mut l, 1, 1, 1, (0, 0));
    won(&mut l, 2, 1, 1, (0, 0));
    won(&mut l, 3, 4, 0, (0, 0));
    won(&mut l, 4, 4, 0, (0, 0));
    won(&mut l, 5, 2, 1, (0, 0));
    won(&mut l, 6, 3, 0, (0, 0));
    won(&mut l, 7, 2, 0, (0, 0));
    won(&mut l, 8, 3, 0, (0, 0));
    assert!(row(&l, 4).map_diff > row(&l, 1).map_diff);
    assert!(row(&l, 3).map_diff > row(&l, 2).map_diff);
    assert_eq!(order(&l), [1, 4, 2, 3, 5]);
    assert_eq!(
        ahead(&l),
        [
            Some(HeadToHead),
            Some(HeadToHead),
            Some(HeadToHead),
            Some(Points),
            None
        ]
    );
}

#[test]
fn head_to_head_needs_everyone_to_have_met() {
    // 1, 2 e 3 com um ponto; 1 venceu 2, mas 3 não enfrentou nenhum dos
    // dois (no suíço, o comum): sem confronto direto, o saldo de mapas.
    let mut l = league(ROUND_ROBIN, 5, &[(1, 2), (3, 4), (2, 5)], &[]);
    won(&mut l, 1, 1, 1, (0, 0));
    won(&mut l, 2, 3, 0, (0, 0));
    won(&mut l, 3, 2, 0, (0, 0));
    let top: Vec<i64> = order(&l)[..3].to_vec();
    assert_eq!(top, [3, 1, 2]);
    assert_eq!(ahead(&l)[0], Some(MapDiff));
}

#[test]
fn frags_then_seed() {
    // 2 e 3 vencem 2 x 1 quem não joga entre eles; 3 com mais frags.
    let mut l = league(ROUND_ROBIN, 5, &[(2, 4), (3, 5), (1, 4)], &[]);
    won(&mut l, 1, 2, 1, (40, 39));
    won(&mut l, 2, 3, 1, (50, 30));
    won(&mut l, 3, 1, 1, (40, 39));
    // 1 e 2 empatam em tudo (pontos, saldo de mapas, de frags): o seed. 4
    // perdeu duas por 2 x 1, 5 uma.
    assert_eq!(order(&l), [3, 1, 2, 5, 4]);
    assert_eq!(
        ahead(&l),
        [
            Some(FragDiff),
            Some(Seed),
            Some(Points),
            Some(MapDiff),
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
fn walkover_is_a_clean_win_without_frags() {
    let mut l = league(ROUND_ROBIN, 2, &[(1, 2)], &[]);
    l.report(StageReport {
        report: MatchReport {
            match_id: mid(1),
            outcome: Outcome::Walkover { winner: p(2) },
        },
        frags: Some(SeriesFrags {
            winner: 10,
            loser: 3,
        }),
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
    assert_eq!((r.points, r.byes, r.played), (0, 0, 0));
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
