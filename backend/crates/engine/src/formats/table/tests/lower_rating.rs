//! O menor rating: o rating congelado de cada participante, o menor à frente,
//! só depois da primeira série decidida e com o rating de todos os empatados.

use super::*;

fn rules(list: &[Criterion]) -> Rules {
    Rules::round_robin(Tiebreakers::new(list).unwrap())
}

fn rated(l: &mut League, ratings: &[(i64, i32)]) {
    let ratings: Vec<(ParticipantId, i32)> = ratings.iter().map(|&(n, r)| (p(n), r)).collect();
    l.set_ratings(&ratings);
    l.refresh();
}

/// 1 vence 2 por 2 x 0 e 3 vence 4 por 2 x 1: 1 e 3 com saldo +1, 2 e 4 com
/// −1, sem confronto entre os empatados. Saldo de mapas: 1 = +2, 3 = +1,
/// 4 = −1, 2 = −2.
fn two_pairs(list: &[Criterion], ratings: &[(i64, i32)]) -> League {
    let mut l = league(rules(list), 4, &[(1, 2), (3, 4)], &[]);
    won(&mut l, 1, 1, 0, (10, 0));
    won(&mut l, 2, 3, 1, (10, 0));
    rated(&mut l, ratings);
    l
}

#[test]
fn the_lower_rating_goes_ahead() {
    let l = two_pairs(
        &[LowerRating],
        &[(1, 1600), (2, 1400), (3, 1500), (4, 1500)],
    );
    assert_eq!(order(&l), [3, 1, 2, 4]);
    assert_eq!(
        ahead(&l),
        [Some(LowerRating), Some(SeriesDiff), Some(LowerRating), None]
    );
    assert_eq!(
        (row(&l, 3).rating, row(&l, 1).rating),
        (Some(1500), Some(1600))
    );
}

#[test]
fn an_equal_rating_goes_to_the_next_criterion() {
    let ratings = [(1, 1500), (2, 1500), (3, 1500), (4, 1500)];
    let l = two_pairs(&[LowerRating, MapDiff], &ratings);
    assert_eq!(order(&l), [1, 3, 4, 2]);
    assert_eq!(
        ahead(&l),
        [Some(MapDiff), Some(SeriesDiff), Some(MapDiff), None]
    );
}

#[test]
fn the_chosen_position_holds() {
    // 3 tem o rating menor, 1 o saldo de mapas melhor: decide o que vem antes.
    let ratings = [(1, 1600), (2, 1500), (3, 1400), (4, 1500)];
    let rating_first = two_pairs(&[LowerRating, MapDiff], &ratings);
    assert_eq!(order(&rating_first)[..2], [3, 1]);
    assert_eq!(ahead(&rating_first)[0], Some(LowerRating));
    let maps_first = two_pairs(&[MapDiff, LowerRating], &ratings);
    assert_eq!(order(&maps_first)[..2], [1, 3]);
    assert_eq!(ahead(&maps_first)[0], Some(MapDiff));
}

#[test]
fn off_by_default_and_without_it_the_rating_is_not_shown() {
    let ratings = [(1, 1600), (2, 1400), (3, 1500), (4, 1500)];
    let mut l = league(ROUND_ROBIN, 4, &[(1, 2), (3, 4)], &[]);
    won(&mut l, 1, 1, 0, (10, 0));
    won(&mut l, 2, 3, 1, (10, 0));
    rated(&mut l, &ratings);
    assert_eq!(order(&l), [1, 3, 4, 2]);
    assert!(l.table().iter().all(|r| r.rating.is_none()));
}

#[test]
fn before_any_series_the_seed_order_holds() {
    let mut l = league(rules(&[LowerRating]), 4, &[(1, 2), (3, 4)], &[]);
    rated(&mut l, &[(1, 1600), (2, 1500), (3, 1400), (4, 1300)]);
    assert_eq!(order(&l), [1, 2, 3, 4]);
    assert_eq!(ahead(&l), [Some(Seed), Some(Seed), Some(Seed), None]);
}

#[test]
fn a_missing_rating_falls_to_the_next_criterion() {
    // 3 sem rating: entre 1 e 3 vale o saldo de mapas; entre 2 e 4, o rating.
    let l = two_pairs(&[LowerRating, MapDiff], &[(1, 1600), (2, 1500), (4, 1400)]);
    assert_eq!(order(&l), [1, 3, 4, 2]);
    assert_eq!(
        ahead(&l),
        [Some(MapDiff), Some(SeriesDiff), Some(LowerRating), None]
    );
}

#[test]
fn ids_outside_the_stage_are_ignored() {
    let l = two_pairs(&[LowerRating], &[(9, 1000), (1, 1600), (3, 1500)]);
    assert_eq!(order(&l)[..2], [3, 1]);
}

#[test]
fn head_to_head_comes_back_after_the_lower_rating() {
    // Ciclo 3 > 2 > 1 > 3, todos 2 x 0, e os três vencem o 4: o confronto
    // direto não separa os três; o rating põe o 1 na frente, e 2 e 3, com o
    // mesmo rating, voltam ao confronto direto, que 3 venceu.
    let pairs = [(3, 2), (2, 1), (1, 3), (1, 4), (2, 4), (3, 4)];
    let mut l = league(rules(&[HeadToHead, LowerRating]), 4, &pairs, &[]);
    for (index, winner) in [(1, 3), (2, 2), (3, 1), (4, 1), (5, 2), (6, 3)] {
        won(&mut l, index, winner, 0, (10, 0));
    }
    rated(&mut l, &[(1, 1400), (2, 1500), (3, 1500), (4, 1300)]);
    assert_eq!(order(&l), [1, 3, 2, 4]);
    assert_eq!(
        ahead(&l),
        [Some(LowerRating), Some(HeadToHead), Some(SeriesDiff), None]
    );
}
