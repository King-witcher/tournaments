//! Posicionamento de seeds numa chave de eliminação.

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
