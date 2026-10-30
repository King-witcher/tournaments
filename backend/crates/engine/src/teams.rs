//! Formação dos times do TDM, antes do seeding: quem fica de reserva e os
//! dois sorteios (aleatório e balanceado pelo rating).
//!
//! Regras:
//! - **Reservas**: dos elegíveis (na ordem de inscrição), os últimos
//!   `n mod tamanho` ficam sem time; o resto forma `⌊n/tamanho⌋` times
//!   completos. A organização reajusta depois, à mão.
//! - **Aleatório**: os que jogam são embaralhados pela `seed` (o
//!   embaralhamento do seeding, ChaCha8, com um propósito próprio na chave:
//!   a mesma `seed` não dá a ordem do seeding) e cortados em times na ordem;
//!   o capitão é o primeiro de cada time, ou seja, sorteado.
//! - **Balanceado**: minimiza a variância das médias de rating dos times
//!   (com os times do mesmo tamanho, a soma dos quadrados das somas; o desvio
//!   padrão dos jogadores não serve de critério, porque não muda com a
//!   divisão). Parte do zigue-zague pelo rating (1º, 2º, ..., k-ésimo time, e
//!   de volta do k-ésimo ao 1º), do maior rating para o menor, e troca pares
//!   de jogadores entre times, sempre a troca que mais reduz a variância,
//!   até nenhuma reduzir. Recomeça `RESTARTS` vezes de uma divisão
//!   embaralhada pela `seed` e fica com a melhor; no empate de variância,
//!   a de menor diferença entre o maior e o menor time, e a que veio
//!   primeiro (o zigue-zague antes). Os empates de rating (comuns: quem não
//!   tem rating entra com o inicial) saem na ordem embaralhada pela `seed`
//!   (o mesmo embaralhamento do aleatório). O capitão é o de maior rating do
//!   time, e os times saem na ordem dos capitães. Nunca é pior que o
//!   zigue-zague, e a diferença entre as somas de dois times nunca passa da
//!   diferença entre o maior e o menor rating (se passasse, trocar o maior
//!   do time mais forte pelo menor do mais fraco reduziria a variância).
//! - Mesma entrada e mesma `seed`, mesmos times, em qualquer versão: os
//!   testes fixam o resultado.
//! - Erros: `InvalidTeamSize` (tamanho 0) e `DuplicatePlayer` (o mesmo
//!   jogador duas vezes na entrada).

use std::collections::{HashMap, HashSet};

use crate::{
    core::PlayerId,
    seeding::{Purpose, shuffle},
};

/// Um time formado: o capitão é o primeiro de `players`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormedTeam {
    pub players: Vec<PlayerId>,
}

impl FormedTeam {
    pub fn captain(&self) -> PlayerId {
        self.players[0]
    }
}

/// Os times e quem ficou de reserva (na ordem de inscrição).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Formation {
    pub teams: Vec<FormedTeam>,
    pub reserves: Vec<PlayerId>,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum FormationError {
    #[error("tamanho de time inválido: {0}")]
    InvalidTeamSize(usize),
    #[error("jogador repetido: {0:?}")]
    DuplicatePlayer(PlayerId),
}

/// Separa os reservas padrão: de `players` na ordem de inscrição, os últimos
/// `len mod team_size` (com `team_size` 0, todos).
pub fn split_reserves<T: Copy>(players: &[T], team_size: usize) -> (Vec<T>, Vec<T>) {
    let playing = match team_size {
        0 => 0,
        size => players.len() / size * size,
    };
    (players[..playing].to_vec(), players[playing..].to_vec())
}

/// Confere a entrada dos sorteios.
fn check(
    players: impl IntoIterator<Item = PlayerId>,
    team_size: usize,
) -> Result<(), FormationError> {
    if team_size == 0 {
        return Err(FormationError::InvalidTeamSize(team_size));
    }
    let mut seen = HashSet::new();
    for p in players {
        if !seen.insert(p) {
            return Err(FormationError::DuplicatePlayer(p));
        }
    }
    Ok(())
}

/// Sorteio aleatório: `players` na ordem de inscrição.
pub fn random(
    players: &[PlayerId],
    team_size: usize,
    seed: u64,
) -> Result<Formation, FormationError> {
    check(players.iter().copied(), team_size)?;
    let (playing, reserves) = split_reserves(players, team_size);
    let shuffled = shuffle(&playing, seed, Purpose::Teams);
    Ok(Formation {
        teams: shuffled
            .chunks(team_size)
            .map(|c| FormedTeam {
                players: c.to_vec(),
            })
            .collect(),
        reserves,
    })
}

/// Quantas divisões embaralhadas o sorteio balanceado tenta além do
/// zigue-zague.
const RESTARTS: u64 = 8;

/// Os times como posições na lista dos jogadores ordenada do maior rating
/// para o menor; o primeiro de cada time é o capitão.
type Split = Vec<Vec<usize>>;

/// A qualidade de uma divisão, quanto menor melhor: a soma dos quadrados das
/// somas de rating dos times (a variância das médias, a menos de uma
/// constante, com os times do mesmo tamanho) e, no empate, a diferença entre
/// o maior e o menor time.
fn cost(split: &Split, rating: &[i64]) -> (i64, i64) {
    let sums: Vec<i64> = split
        .iter()
        .map(|t| t.iter().map(|&i| rating[i]).sum())
        .collect();
    let spread = sums.iter().max().unwrap_or(&0) - sums.iter().min().unwrap_or(&0);
    (sums.iter().map(|s| s * s).sum(), spread)
}

/// O zigue-zague: o 1º, 2º, ..., k-ésimo time e de volta do k-ésimo ao 1º.
fn zigzag(players: usize, count: usize) -> Split {
    let mut teams = vec![Vec::new(); count];
    for i in 0..players {
        let (round, pos) = (i / count, i % count);
        let team = if round.is_multiple_of(2) {
            pos
        } else {
            count - 1 - pos
        };
        teams[team].push(i);
    }
    teams
}

/// Troca o par de jogadores de dois times que mais reduz a variância, até
/// nenhuma troca reduzir. Trocar `d` de rating do time A (soma `sa`) para o B
/// (soma `sb`) muda a soma dos quadrados em `2d(d - (sa - sb))`: só vale
/// quando é negativa, então termina (inteiros que só diminuem).
fn improve(split: &mut Split, rating: &[i64]) {
    loop {
        let sums: Vec<i64> = split
            .iter()
            .map(|t| t.iter().map(|&i| rating[i]).sum())
            .collect();
        let mut best: Option<(i64, usize, usize, usize, usize)> = None;
        for a in 0..split.len() {
            for b in a + 1..split.len() {
                for (x, &i) in split[a].iter().enumerate() {
                    for (y, &j) in split[b].iter().enumerate() {
                        let d = rating[i] - rating[j];
                        let change = 2 * d * (d - (sums[a] - sums[b]));
                        if change < 0 && best.is_none_or(|(c, ..)| change < c) {
                            best = Some((change, a, b, x, y));
                        }
                    }
                }
            }
        }
        let Some((_, a, b, x, y)) = best else { return };
        let (low, high) = split.split_at_mut(b);
        std::mem::swap(&mut low[a][x], &mut high[0][y]);
    }
}

/// Sorteio balanceado: `players` na ordem de inscrição, cada um com o rating
/// (quem não tem, com o inicial).
pub fn balanced(
    players: &[(PlayerId, i32)],
    team_size: usize,
    seed: u64,
) -> Result<Formation, FormationError> {
    check(players.iter().map(|(p, _)| *p), team_size)?;
    let (playing, reserves) = split_reserves(players, team_size);
    let rating: HashMap<PlayerId, i32> = playing.iter().copied().collect();
    let ids: Vec<PlayerId> = playing.iter().map(|(p, _)| *p).collect();
    let mut order = shuffle(&ids, seed, Purpose::Teams);
    // Estável: os empates ficam na ordem sorteada.
    order.sort_by_key(|p| std::cmp::Reverse(rating[p]));
    let count = order.len() / team_size;
    let by_rank: Vec<i64> = order.iter().map(|p| i64::from(rating[p])).collect();

    let mut best = zigzag(order.len(), count);
    improve(&mut best, &by_rank);
    let ranks: Vec<usize> = (0..order.len()).collect();
    for restart in 1..=RESTARTS {
        let mut split: Split = shuffle(&ranks, seed.wrapping_add(restart), Purpose::Teams)
            .chunks(team_size)
            .map(<[usize]>::to_vec)
            .collect();
        improve(&mut split, &by_rank);
        if cost(&split, &by_rank) < cost(&best, &by_rank) {
            best = split;
        }
    }

    // O capitão é o de maior rating (a menor posição); os times, na ordem dos capitães.
    for team in &mut best {
        team.sort_unstable();
    }
    best.sort_unstable();
    Ok(Formation {
        teams: best
            .into_iter()
            .map(|team| FormedTeam {
                players: team.into_iter().map(|i| order[i]).collect(),
            })
            .collect(),
        reserves: reserves.into_iter().map(|(p, _)| p).collect(),
    })
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::{core::ParticipantId, seeding::random_order};

    fn ids(n: i64) -> Vec<PlayerId> {
        (1..=n).map(PlayerId).collect()
    }

    fn raw(f: &Formation) -> Vec<Vec<i64>> {
        f.teams
            .iter()
            .map(|t| t.players.iter().map(|p| p.0).collect())
            .collect()
    }

    fn rated(ratings: &[i32]) -> Vec<(PlayerId, i32)> {
        ratings
            .iter()
            .zip(1..)
            .map(|(r, id)| (PlayerId(id), *r))
            .collect()
    }

    #[test]
    fn the_last_registered_are_the_reserves() {
        let f = random(&ids(7), 2, 1).unwrap();
        assert_eq!(f.reserves, vec![PlayerId(7)]);
        assert_eq!(f.teams.len(), 3);
    }

    #[test]
    fn fewer_players_than_a_team_are_all_reserves() {
        for f in [
            random(&ids(3), 4, 1).unwrap(),
            balanced(&rated(&[1500, 1600, 1700]), 4, 1).unwrap(),
        ] {
            assert!(f.teams.is_empty());
            assert_eq!(f.reserves, ids(3));
        }
    }

    #[test]
    fn a_single_team() {
        let f = balanced(&rated(&[1500, 1900, 1700, 1600, 1800]), 4, 3).unwrap();
        // Um time só: todos em ordem de rating, o último inscrito de reserva.
        assert_eq!(raw(&f), [[2, 3, 4, 1]]);
        assert_eq!(f.reserves, [PlayerId(5)]);
        assert_eq!(random(&ids(4), 4, 3).unwrap().teams.len(), 1);
    }

    #[test]
    fn team_size_one_is_one_team_per_player() {
        let f = balanced(&rated(&[1500, 1900, 1700]), 1, 5).unwrap();
        assert_eq!(raw(&f), [[2], [3], [1]]);
        assert!(f.reserves.is_empty());
        let f = random(&ids(3), 1, 5).unwrap();
        assert_eq!(f.teams.len(), 3);
    }

    #[test]
    fn team_size_zero_is_refused() {
        assert_eq!(
            random(&ids(4), 0, 1),
            Err(FormationError::InvalidTeamSize(0))
        );
        assert_eq!(
            balanced(&rated(&[1500, 1500]), 0, 1),
            Err(FormationError::InvalidTeamSize(0))
        );
    }

    #[test]
    fn a_repeated_player_is_refused() {
        let twice = [PlayerId(1), PlayerId(2), PlayerId(1), PlayerId(3)];
        assert_eq!(
            random(&twice, 2, 1),
            Err(FormationError::DuplicatePlayer(PlayerId(1)))
        );
        let twice: Vec<(PlayerId, i32)> = twice.iter().map(|p| (*p, 1500)).collect();
        assert_eq!(
            balanced(&twice, 2, 1),
            Err(FormationError::DuplicatePlayer(PlayerId(1)))
        );
    }

    #[test]
    fn random_is_pinned() {
        // Sorteios gravados dependem deste resultado.
        assert_eq!(
            raw(&random(&ids(8), 2, 2026).unwrap()),
            [[4, 7], [1, 5], [8, 6], [3, 2]]
        );
    }

    #[test]
    fn balanced_is_pinned() {
        // Sorteios gravados dependem deste resultado: empates no 1700 e no
        // 1500 (o inicial), saindo na ordem da seed.
        let players = rated(&[1500, 1700, 1500, 1900, 1700, 1500, 1200, 1500]);
        assert_eq!(
            raw(&balanced(&players, 2, 2026).unwrap()),
            [[4, 7], [5, 3], [2, 6], [1, 8]]
        );
    }

    /// A soma de rating de cada time, na ordem dos times.
    fn sums(f: &Formation, rated: &[(PlayerId, i32)]) -> Vec<i64> {
        let of = |p: &PlayerId| i64::from(rated.iter().find(|(q, _)| q == p).unwrap().1);
        f.teams
            .iter()
            .map(|t| t.players.iter().map(of).sum())
            .collect()
    }

    fn squares(sums: &[i64]) -> i64 {
        sums.iter().map(|s| s * s).sum()
    }

    #[test]
    fn balanced_beats_the_snake_when_it_can() {
        // O zigue-zague dá somas de 4700, 4600, 4100 e 4000. A média é 4350 e
        // as somas são múltiplas de 100, então o melhor são duas de 4300 e
        // duas de 4400, e as trocas chegam lá. Sorteios gravados dependem
        // deste resultado.
        let players = rated(&[
            2400, 2300, 1800, 1700, 1600, 1500, 1400, 1300, 1000, 900, 800, 700,
        ]);
        let f = balanced(&players, 3, 7).unwrap();
        assert_eq!(raw(&f), [[1, 9, 10], [2, 7, 12], [3, 4, 11], [5, 6, 8]]);
        assert_eq!(sums(&f, &players), [4300, 4400, 4300, 4400]);
        let snake = zigzag(12, 4);
        let by_rank: Vec<i64> = players.iter().map(|(_, r)| i64::from(*r)).collect();
        assert!(cost(&snake, &by_rank).0 > squares(&sums(&f, &players)));
        assert_eq!(f.teams[0].captain(), PlayerId(1));
    }

    #[test]
    fn the_reserves_do_not_count_in_the_balance() {
        // 11 inscritos, times de 3: os dois últimos, com ratings extremos,
        // ficam de reserva, e os 9 que jogam se equilibram entre si.
        let mut ratings = vec![2000, 1900, 1800, 1700, 1600, 1500, 1400, 1300, 1200];
        ratings.extend([3000, 100]);
        let players = rated(&ratings);
        let f = balanced(&players, 3, 11).unwrap();
        assert_eq!(f.reserves, [PlayerId(10), PlayerId(11)]);
        assert_eq!(f.teams.len(), 3);
        // A soma de todos é 14400: 4800 para cada, e as trocas chegam lá.
        assert_eq!(sums(&f, &players), [4800, 4800, 4800]);
    }

    #[test]
    fn the_same_seed_gives_the_same_teams() {
        let players = rated(&[1500, 1500, 1800, 1200, 1500, 1700, 1500, 1300, 1500, 1600]);
        for seed in [0, 5, 2026] {
            assert_eq!(
                balanced(&players, 3, seed).unwrap(),
                balanced(&players, 3, seed).unwrap()
            );
        }
    }

    #[test]
    fn teams_and_seeding_draw_differently_with_the_same_seed() {
        // O mesmo conjunto e a mesma seed: o seeding e os times não saem na
        // mesma ordem (propósitos diferentes na chave do ChaCha8).
        let n = 16;
        for seed in [0, 1, 2026, u64::MAX] {
            let seeding: Vec<i64> =
                random_order(&(1..=n).map(ParticipantId).collect::<Vec<_>>(), seed)
                    .into_iter()
                    .map(|p| p.0)
                    .collect();
            let teams: Vec<i64> = random(&ids(n), 1, seed)
                .unwrap()
                .teams
                .iter()
                .map(|t| t.players[0].0)
                .collect();
            assert_ne!(seeding, teams, "seed {seed}");
        }
    }

    #[test]
    fn balanced_snakes_by_rating() {
        // 6 jogadores, 3 times de 2: 1º, 2º, 3º e de volta.
        let players = rated(&[1900, 1800, 1700, 1600, 1500, 1400]);
        let f = balanced(&players, 2, 9).unwrap();
        assert_eq!(raw(&f), [[1, 6], [2, 5], [3, 4]]);
        // O capitão é o de maior rating.
        assert_eq!(f.teams[0].captain(), PlayerId(1));
    }

    #[test]
    fn ties_follow_the_seeded_shuffle() {
        // Todos empatados: a ordem do zigue-zague é a do embaralhamento dos
        // times com a mesma seed, para qualquer seed.
        let players: Vec<(PlayerId, i32)> = ids(8).into_iter().map(|p| (p, 1500)).collect();
        for seed in [0, 1, 7, 2026, u64::MAX] {
            let order = shuffle(&ids(8), seed, Purpose::Teams);
            let expected: Vec<Vec<PlayerId>> =
                (0..4).map(|t| vec![order[t], order[7 - t]]).collect();
            let got: Vec<Vec<PlayerId>> = balanced(&players, 2, seed)
                .unwrap()
                .teams
                .into_iter()
                .map(|t| t.players)
                .collect();
            assert_eq!(got, expected, "seed {seed}");
        }
    }

    #[test]
    fn mixed_ties_keep_the_rating_order() {
        // Empates misturados com ratings distintos: os empatados trocam de
        // lugar entre si conforme a seed, mas o rating de cada posição do
        // zigue-zague é sempre o mesmo.
        let players = rated(&[1500, 1800, 1500, 1200, 1800, 1500, 2000, 1200]);
        let of = |p: &PlayerId| players.iter().find(|(q, _)| q == p).unwrap().1;
        for seed in 0..20 {
            let f = balanced(&players, 2, seed).unwrap();
            let shape: Vec<Vec<i32>> = f
                .teams
                .iter()
                .map(|t| t.players.iter().map(of).collect())
                .collect();
            assert_eq!(
                shape,
                [[2000, 1200], [1800, 1200], [1800, 1500], [1500, 1500]],
                "seed {seed}"
            );
            assert_eq!(f.teams[0].captain(), PlayerId(7));
        }
    }

    proptest! {
        #[test]
        fn everyone_once_full_teams(
            n in 0usize..40,
            size in 1usize..=4,
            seed in any::<u64>(),
            ratings in proptest::collection::vec(1000i32..2500, 40),
        ) {
            let players = ids(n as i64);
            let rated: Vec<(PlayerId, i32)> =
                players.iter().copied().zip(ratings.iter().copied()).collect();
            for f in [random(&players, size, seed).unwrap(), balanced(&rated, size, seed).unwrap()] {
                prop_assert_eq!(f.teams.len(), n / size);
                prop_assert!(f.teams.iter().all(|t| t.players.len() == size));
                prop_assert_eq!(&f.reserves[..], &players[n / size * size..]);
                let mut all: Vec<PlayerId> =
                    f.teams.iter().flat_map(|t| t.players.clone()).collect();
                all.extend(&f.reserves);
                all.sort();
                prop_assert_eq!(&all, &players);
            }
        }

        #[test]
        fn balanced_is_never_worse_than_the_snake(
            ratings in proptest::collection::vec(1000i32..2500, 2..40),
            size in 1usize..=4,
            seed in any::<u64>(),
        ) {
            let rated = rated(&ratings);
            let f = balanced(&rated, size, seed).unwrap();
            let playing = ratings.len() / size * size;
            prop_assume!(playing > 0);
            let mut by_rank: Vec<i64> =
                ratings[..playing].iter().map(|r| i64::from(*r)).collect();
            by_rank.sort_by(|a, b| b.cmp(a));
            let snake = cost(&zigzag(playing, playing / size), &by_rank);
            let sums = sums(&f, &rated);
            let got = (
                squares(&sums),
                sums.iter().max().unwrap() - sums.iter().min().unwrap(),
            );
            prop_assert!(got <= snake, "{got:?} {snake:?}");
        }

        #[test]
        fn balanced_sums_stay_within_the_rating_range(
            ratings in proptest::collection::vec(1000i32..2500, 4..40),
            size in 2usize..=4,
            seed in any::<u64>(),
        ) {
            let rated = rated(&ratings);
            let f = balanced(&rated, size, seed).unwrap();
            prop_assume!(!f.teams.is_empty());
            let of = |p: &PlayerId| rated.iter().find(|(q, _)| q == p).unwrap().1;
            let sums: Vec<i32> =
                f.teams.iter().map(|t| t.players.iter().map(of).sum()).collect();
            let playing: Vec<i32> = f
                .teams
                .iter()
                .flat_map(|t| t.players.iter().map(of))
                .collect();
            let range = playing.iter().max().unwrap() - playing.iter().min().unwrap();
            let spread = sums.iter().max().unwrap() - sums.iter().min().unwrap();
            prop_assert!(spread <= range, "{sums:?} {range}");
            // O capitão tem o maior rating do time.
            for t in &f.teams {
                prop_assert!(t.players.iter().all(|p| of(p) <= of(&t.captain())));
            }
        }
    }
}
