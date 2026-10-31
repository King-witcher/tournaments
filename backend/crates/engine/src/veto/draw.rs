//! Os sorteios do veto, reproduzíveis pela seed da série: cada sorteio tem a
//! própria sequência do ChaCha8, separada pelo propósito, pela partida e por um
//! índice (os 8 bytes da seed, little-endian, no início da chave de 32
//! bytes; depois o propósito, a partida e o índice; o resto zero). Mudar qualquer
//! passo muda os sorteios já gravados: os testes fixam alguns resultados.

use chacha20::ChaCha8Rng;
use rand_core::SeedableRng;

use crate::seeding::below;

/// Para que é o sorteio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purpose {
    /// Os mapas oferecidos na votação de uma partida.
    Offer = 1,
    /// O desempate entre os dois votos diferentes.
    Flip = 2,
    /// O mapa da partida no modo aleatório.
    Draw = 3,
    /// A escolha de quem deixou o tempo do passo acabar.
    Timeout = 4,
    /// Nos times, o campeão de quem não escolheu na janela simultânea.
    TeamTimeout = 5,
}

fn rng(seed: u64, purpose: Purpose, game: u8, index: u32) -> ChaCha8Rng {
    let mut key = [0u8; 32];
    key[..8].copy_from_slice(&seed.to_le_bytes());
    key[8] = purpose as u8;
    key[9] = game;
    key[10..14].copy_from_slice(&index.to_le_bytes());
    ChaCha8Rng::from_seed(key)
}

/// Um índice uniforme em `0..n` (`n > 0`).
pub fn index(seed: u64, purpose: Purpose, game: u8, at: u32, n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    below(&mut rng(seed, purpose, game, at), n as u64) as usize
}

/// `k` dos `n` índices `0..n`, sem repetir, em ordem crescente (Fisher–Yates
/// parcial).
pub fn choose(seed: u64, purpose: Purpose, game: u8, n: usize, k: usize) -> Vec<usize> {
    let mut rng = rng(seed, purpose, game, 0);
    let mut all: Vec<usize> = (0..n).collect();
    let k = k.min(n);
    for i in 0..k {
        let j = i + below(&mut rng, (n - i) as u64) as usize;
        all.swap(i, j);
    }
    let mut chosen = all[..k].to_vec();
    chosen.sort_unstable();
    chosen
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_draws_the_same() {
        assert_eq!(
            choose(7, Purpose::Offer, 1, 7, 3),
            choose(7, Purpose::Offer, 1, 7, 3)
        );
        assert_ne!(
            choose(7, Purpose::Offer, 1, 7, 3),
            choose(7, Purpose::Offer, 2, 7, 3)
        );
    }

    #[test]
    fn choose_is_a_subset() {
        for seed in 0..50 {
            let c = choose(seed, Purpose::Offer, 1, 5, 3);
            assert_eq!(c.len(), 3);
            assert!(c.windows(2).all(|w| w[0] < w[1]) && c[2] < 5);
        }
        assert_eq!(choose(1, Purpose::Offer, 1, 2, 3), [0, 1]);
    }

    #[test]
    fn draws_are_pinned() {
        // Sorteios já gravados dependem destes resultados.
        assert_eq!(choose(2026, Purpose::Offer, 1, 7, 3), [0, 2, 5]);
        assert_eq!(index(2026, Purpose::Timeout, 1, 4, 17), 8);
    }
}
