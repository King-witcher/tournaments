//! Seeds: o sorteio da ordem e o posicionamento numa chave de eliminação.

use chacha20::ChaCha8Rng;
use rand_core::{Rng, SeedableRng};

use crate::core::ParticipantId;

/// Ordem sorteada de `participants` (o primeiro é o seed 1), reproduzível: o
/// mesmo conjunto com a mesma `seed` dá sempre a mesma ordem, em qualquer
/// máquina e versão. A ordem de entrada não importa (é ordenada antes).
///
/// Embaralhamento de Fisher–Yates com ChaCha8 sobre a `seed` (os 8 bytes
/// little-endian no início da chave de 32 bytes, o resto zero) e sorteio de
/// índice por rejeição, sem viés. Mudar qualquer passo muda os sorteios já
/// salvos: o teste `draw_is_pinned` fixa o resultado.
pub fn random_order(participants: &[ParticipantId], seed: u64) -> Vec<ParticipantId> {
    let mut order = participants.to_vec();
    order.sort_unstable();
    let mut key = [0u8; 32];
    key[..8].copy_from_slice(&seed.to_le_bytes());
    let mut rng = ChaCha8Rng::from_seed(key);
    for i in (1..order.len()).rev() {
        let j = below(&mut rng, i as u64 + 1) as usize;
        order.swap(i, j);
    }
    order
}

/// Inteiro uniforme em `0..n`: descarta o resto que favoreceria os menores.
fn below(rng: &mut ChaCha8Rng, n: u64) -> u64 {
    let limit = u64::MAX - u64::MAX % n;
    loop {
        let x = rng.next_u64();
        if x < limit {
            return x % n;
        }
    }
}

/// Tamanho da chave para `participants`: a menor potência de 2 que comporta
/// todos (mínimo 2). As posições que sobram viram byes dos melhores seeds.
pub fn bracket_size(participants: usize) -> usize {
    participants.max(2).next_power_of_two()
}

/// Ordem padrão de seeds (começando em 1) para uma chave de `size` posições,
/// `size` potência de 2. As posições `[0, 1]` formam a partida 1, `[2, 3]` a
/// partida 2, e assim por diante.
///
/// Com todos os favoritos vencendo, os seeds 1 e 2 só se encontram na final,
/// 1–4 nas semifinais, 1–8 nas quartas. Seeds maiores que o número de
/// participantes são byes, e caem contra os melhores seeds.
pub fn bracket_order(size: usize) -> Vec<usize> {
    assert!(
        size >= 2 && size.is_power_of_two(),
        "tamanho de chave inválido: {size}"
    );
    let mut order = vec![1];
    while order.len() < size {
        let next_len = order.len() * 2;
        order = order
            .iter()
            .flat_map(|&seed| [seed, next_len + 1 - seed])
            .collect();
    }
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(range: std::ops::RangeInclusive<i64>) -> Vec<ParticipantId> {
        range.map(ParticipantId).collect()
    }

    #[test]
    fn same_seed_same_order() {
        let all = ids(1..=16);
        assert_eq!(random_order(&all, 42), random_order(&all, 42));
        assert_ne!(random_order(&all, 42), random_order(&all, 43));
    }

    #[test]
    fn input_order_does_not_matter() {
        let mut reversed = ids(1..=16);
        reversed.reverse();
        assert_eq!(random_order(&reversed, 7), random_order(&ids(1..=16), 7));
    }

    #[test]
    fn draw_is_a_permutation() {
        for n in [0, 1, 2, 3, 17, 64] {
            let all = ids(1..=n);
            let mut drawn = random_order(&all, n as u64);
            drawn.sort();
            assert_eq!(drawn, all);
        }
    }

    #[test]
    fn draw_is_pinned() {
        // Sorteios já salvos dependem deste resultado: se ele mudar, a seed
        // gravada deixa de reproduzir a ordem.
        let drawn: Vec<i64> = random_order(&ids(1..=8), 2026)
            .into_iter()
            .map(|p| p.0)
            .collect();
        assert_eq!(drawn, [8, 7, 3, 1, 6, 4, 5, 2]);
    }

    #[test]
    fn every_position_is_reachable() {
        // Sem viés grosseiro: em 2000 sorteios de 4, cada um cai em cada
        // posição perto de 500 vezes.
        let all = ids(1..=4);
        let mut counts = [[0u32; 4]; 4];
        for seed in 0..2000 {
            for (pos, p) in random_order(&all, seed).into_iter().enumerate() {
                counts[(p.0 - 1) as usize][pos] += 1;
            }
        }
        for row in counts {
            for c in row {
                assert!((400..600).contains(&c), "{counts:?}");
            }
        }
    }

    #[test]
    fn size_rounds_up_to_power_of_two() {
        assert_eq!(bracket_size(0), 2);
        assert_eq!(bracket_size(2), 2);
        assert_eq!(bracket_size(3), 4);
        assert_eq!(bracket_size(8), 8);
        assert_eq!(bracket_size(9), 16);
    }

    #[test]
    fn standard_order_for_eight() {
        assert_eq!(bracket_order(8), vec![1, 8, 4, 5, 2, 7, 3, 6]);
    }

    #[test]
    fn first_round_pairs_sum_to_size_plus_one() {
        for size in [2, 4, 8, 16, 32, 64] {
            let order = bracket_order(size);
            for pair in order.chunks(2) {
                assert_eq!(pair[0] + pair[1], size + 1, "size {size}: {pair:?}");
            }
        }
    }

    #[test]
    fn top_two_seeds_are_in_opposite_halves() {
        for size in [4, 8, 16, 32, 64] {
            let order = bracket_order(size);
            let (top, bottom) = order.split_at(size / 2);
            assert!(top.contains(&1) && bottom.contains(&2), "size {size}");
        }
    }
}
