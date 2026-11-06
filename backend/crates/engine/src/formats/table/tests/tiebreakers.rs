//! Os desempates escolhidos pela fase: o que a lista aceita e como outra
//! ordem, um desligado ou nenhum mudam a tabela.

use super::*;
use crate::formats::StageFormat;
use crate::formats::round_robin::RoundRobinConfig;
use crate::formats::swiss::SwissConfig;

fn round_robin(list: &[Criterion]) -> Rules {
    Rules::round_robin(Tiebreakers::new(list).unwrap())
}

#[test]
fn each_tiebreaker_at_most_once_and_only_tiebreakers() {
    assert_eq!(Tiebreakers::new(&[]), Ok(Tiebreakers::NONE));
    assert_eq!(Tiebreakers::NONE.as_slice(), []);
    assert_eq!(
        Tiebreakers::new(&[HeadToHead, MapDiff, FragDiff]),
        Ok(Tiebreakers::ROUND_ROBIN)
    );
    assert_eq!(
        Tiebreakers::new(&[Buchholz, HeadToHead, MapDiff, FragDiff]),
        Ok(Tiebreakers::SWISS)
    );
    let custom = Tiebreakers::new(&[FragDiff, HeadToHead]).unwrap();
    assert_eq!(custom.as_slice(), [FragDiff, HeadToHead]);
    assert!(custom.contains(HeadToHead) && !custom.contains(MapDiff));

    let invalid = Err(EngineError::InvalidTiebreakers);
    assert_eq!(Tiebreakers::new(&[MapDiff, MapDiff]), invalid);
    let all = [Buchholz, HeadToHead, MapDiff, FragDiff, LowerRating];
    assert_eq!(Tiebreakers::new(&all).unwrap().as_slice(), all);
    assert_eq!(
        Tiebreakers::new(&[all.as_slice(), &[HeadToHead]].concat()),
        invalid
    );
    assert_eq!(Tiebreakers::new(&[LowerRating, LowerRating]), invalid);
    // O primeiro critério e o seed são fixos.
    for fixed in [Points, SeriesDiff, Seed] {
        assert_eq!(Tiebreakers::new(&[HeadToHead, fixed]), invalid, "{fixed:?}");
    }
}

#[test]
fn buchholz_only_in_the_swiss() {
    let buchholz = Tiebreakers::new(&[Buchholz, HeadToHead]).unwrap();
    let rr = StageFormat::RoundRobin(RoundRobinConfig {
        best_of: 3,
        tiebreakers: buchholz,
    });
    assert_eq!(rr.validate(), Err(EngineError::InvalidTiebreakers));
    assert_eq!(rr.worst_case_games(8), Err(EngineError::InvalidTiebreakers));
    for tiebreakers in [buchholz, Tiebreakers::NONE, Tiebreakers::SWISS] {
        let swiss = StageFormat::Swiss(SwissConfig {
            best_of: 3,
            rounds: None,
            tiebreakers,
        });
        assert_eq!(swiss.validate(), Ok(()));
    }
    let none = StageFormat::RoundRobin(RoundRobinConfig {
        best_of: 3,
        tiebreakers: Tiebreakers::NONE,
    });
    assert_eq!(none.validate(), Ok(()));
}

/// 2 e 3 com saldo +1: 2 tem o saldo de mapas melhor (+3 contra +1), e 3
/// venceu 2. 1 e 5 com −1, sem confronto: o saldo de mapas põe o 1 na frente.
/// Na ordem padrão, 3 fica à frente pelo confronto direto
/// (`head_to_head_beats_a_better_map_diff`).
fn map_diff_against_head_to_head(rules: Rules, frags: [(i32, i32); 5]) -> League {
    let pairs = [(2, 3), (2, 4), (2, 5), (1, 3), (3, 4)];
    let mut l = league(rules, 5, &pairs, &[]);
    let results = [(3, 1), (2, 0), (2, 0), (3, 1), (4, 1)];
    for (i, ((winner, loser_maps), f)) in results.iter().zip(frags).enumerate() {
        won(&mut l, i as u16 + 1, *winner, *loser_maps, f);
    }
    l
}

#[test]
fn map_diff_before_head_to_head() {
    let l = map_diff_against_head_to_head(round_robin(&[MapDiff, HeadToHead]), [(0, 0); 5]);
    assert_eq!(order(&l), [2, 3, 4, 1, 5]);
    assert_eq!(
        ahead(&l),
        [
            Some(MapDiff),
            Some(SeriesDiff),
            Some(SeriesDiff),
            Some(MapDiff),
            None
        ]
    );
}

#[test]
fn frag_diff_before_head_to_head() {
    // 2 com mais frags que 3, que venceu a série entre os dois.
    let frags = [(30, 29), (40, 0), (40, 0), (10, 9), (10, 9)];
    let l = map_diff_against_head_to_head(round_robin(&[FragDiff, HeadToHead]), frags);
    assert!(row(&l, 2).frag_diff > row(&l, 3).frag_diff);
    assert_eq!(order(&l)[..2], [2, 3]);
    assert_eq!(ahead(&l)[0], Some(FragDiff));
}

/// Ciclo 3 > 2 > 1 > 3 (2 x 0), e 1, 2 e 3 vencem o 4 (1 por 2 x 0, 2 e 3 por
/// 2 x 1). Saldos de mapas: 1 = +2, 2 = 3 = +1. Os frags favorecem o 2 (+19
/// contra 0), e 3 venceu 2 (`head_to_head_returns_after_map_diff`).
fn cycle(rules: Rules) -> League {
    let pairs = [(3, 2), (2, 1), (1, 3), (1, 4), (2, 4), (3, 4)];
    let mut l = league(rules, 4, &pairs, &[]);
    won(&mut l, 1, 3, 0, (10, 9));
    won(&mut l, 2, 2, 0, (30, 10));
    won(&mut l, 3, 1, 0, (10, 9));
    won(&mut l, 4, 1, 0, (0, 0));
    won(&mut l, 5, 2, 1, (0, 0));
    won(&mut l, 6, 3, 1, (0, 0));
    l
}

#[test]
fn head_to_head_off_does_not_come_back_after_map_diff() {
    // Na ordem padrão, 2 e 3 voltariam ao confronto direto, que 3 venceu.
    let l = cycle(round_robin(&[MapDiff, FragDiff]));
    assert_eq!(order(&l), [1, 2, 3, 4]);
    assert_eq!(
        ahead(&l),
        [Some(MapDiff), Some(FragDiff), Some(SeriesDiff), None]
    );
}

#[test]
fn head_to_head_after_map_diff_still_comes_back_after_frag_diff() {
    // O mesmo ciclo, todas 2 x 0: os mapas empatam, o confronto direto (o
    // ciclo) também, os frags põem o 1 na frente, e 2 e 3 voltam ao confronto
    // direto, que estava antes dos frags na ordem.
    let pairs = [(3, 2), (2, 1), (1, 3), (1, 4), (2, 4), (3, 4)];
    let mut l = league(
        round_robin(&[MapDiff, HeadToHead, FragDiff]),
        4,
        &pairs,
        &[],
    );
    won(&mut l, 1, 3, 0, (10, 9));
    won(&mut l, 2, 2, 0, (10, 9));
    won(&mut l, 3, 1, 0, (10, 9));
    won(&mut l, 4, 1, 0, (50, 0));
    won(&mut l, 5, 2, 0, (0, 0));
    won(&mut l, 6, 3, 0, (0, 0));
    assert_eq!(order(&l), [1, 3, 2, 4]);
    assert_eq!(
        ahead(&l),
        [Some(FragDiff), Some(HeadToHead), Some(SeriesDiff), None]
    );
}

#[test]
fn with_no_tiebreakers_the_seed_decides() {
    let l = cycle(round_robin(&[]));
    assert_eq!(order(&l), [1, 2, 3, 4]);
    assert_eq!(ahead(&l), [Some(Seed), Some(Seed), Some(SeriesDiff), None]);
}

#[test]
fn frags_that_do_not_compare_fall_to_the_next_tiebreaker() {
    // 2 e 3 com saldo +1 e saldo de mapas +2; 2 tem mais frags, mas a série
    // do 3 contra o 1 veio sem frags, então o saldo de frags não vale para os
    // dois, e o confronto direto, depois dele na ordem, decide: 3 venceu 2.
    let pairs = [(2, 3), (2, 4), (2, 5), (1, 3), (3, 4)];
    let mut l = league(round_robin(&[FragDiff, HeadToHead]), 5, &pairs, &[]);
    won(&mut l, 1, 3, 0, (10, 9));
    won(&mut l, 2, 2, 0, (50, 0));
    won(&mut l, 3, 2, 0, (50, 0));
    won_without_frags(&mut l, 4, 3, 0);
    won(&mut l, 5, 4, 0, (0, 0));
    assert_eq!((row(&l, 2).series_diff, row(&l, 3).series_diff), (1, 1));
    assert_eq!((row(&l, 2).map_diff, row(&l, 3).map_diff), (2, 2));
    assert!(!row(&l, 3).frags_complete);
    assert_eq!(order(&l)[..2], [3, 2]);
    assert_eq!(ahead(&l)[0], Some(HeadToHead));
}

#[test]
fn the_swiss_without_buchholz_skips_it() {
    // O cenário de `swiss_bye_is_a_point_and_buchholz_comes_next`: com o
    // Buchholz, 2 (Buchholz 1) fica à frente de 4 e 6 (0). Sem ele, o saldo
    // de mapas põe 1 e 4 (+2) à frente de 2 e 6 (0), e o seed separa cada par.
    let tiebreakers = Tiebreakers::new(&[HeadToHead, MapDiff, FragDiff]).unwrap();
    let pairs = [(1, 2), (4, 5), (2, 3)];
    let mut l = league(Rules::swiss(tiebreakers), 6, &pairs, &[6]);
    won(&mut l, 1, 1, 0, (0, 0));
    won(&mut l, 2, 4, 0, (0, 0));
    won(&mut l, 3, 2, 0, (0, 0));
    assert_eq!(order(&l), [1, 4, 2, 6, 3, 5]);
    assert_eq!(
        ahead(&l),
        [
            Some(Seed),
            Some(MapDiff),
            Some(Seed),
            Some(Points),
            Some(Seed),
            None
        ]
    );
    assert!(l.table().iter().all(|r| r.buchholz.is_none()));
}
