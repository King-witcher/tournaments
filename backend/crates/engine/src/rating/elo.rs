//! Elo com K que decai com os jogos de cada pessoa: quem está começando anda
//! rápido até o rating dele, e o de quem já jogou muito fica estável.
//!
//! - placar esperado do lado A contra o B: `1 / (1 + 10^((B - A) / 400))`,
//!   com A e B a média aritmética dos ratings da escalação de cada lado
//!   naquele mapa (no duelo, o rating de cada jogador; a média de um número
//!   só é ele mesmo, então o duelo dá exatamente o Elo de dois jogadores);
//! - `K(n) = k_min + (k_max - k_min) · e^(-n / k_decay)`, com `n` os jogos
//!   que a pessoa já tinha antes deste (padrão: `32 + 48 · e^(-n/10)`, 80 no
//!   primeiro jogo, ~50 no décimo, ~32 depois de uns 40);
//! - cada jogador do lado vencedor ganha `round(K_i · (1 - E_v))` e cada um
//!   do perdedor perde `round(K_i · (1 - E_v))`, cada um com o próprio K.
//!   Com o mesmo número de jogos (o mesmo K), o que um ganha o outro perde;
//!   com K diferentes (novo contra veterano), não, e mesmo com 100 e 101
//!   jogos (K a ~0,001 um do outro) o arredondamento pode dar 1 ponto de
//!   diferença;
//! - sem piso nem teto: ajustes podem levar o rating abaixo de 100 ou acima de
//!   3000 (as faixas são só do inicial). As contas saturam em vez de estourar.
//!
//! O arredondamento é o de `f64::round` (metade para longe do zero), por
//! evento. `exp` e `powf` vêm da libm da plataforma, que pode diferir no
//! último bit entre o Windows (desenvolvimento) e o Linux (produção); só
//! mudaria um arredondamento caído a ~1e-13 da metade, o que os valores com
//! ratings inteiros praticamente nunca dão (metade exata só sai com ratings
//! iguais e K inteiro, em que as contas são exatas). O teste
//! `regression_values` fixa casos para pegar uma divergência.

use std::ops::RangeInclusive;

use thiserror::Error;

use super::{RatingSystem, Seat};

/// Parâmetros do Elo de uma comunidade. As faixas aceitas estão nas
/// constantes (e repetidas como `check` na tabela da API).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EloConfig {
    /// Rating de quem ainda não jogou.
    pub initial: i32,
    /// K de quem já jogou muito (o piso).
    pub k_min: i32,
    /// K do primeiro jogo.
    pub k_max: i32,
    /// Em quantos jogos a distância de K até o piso cai para ~37% (1/e).
    pub k_decay: i32,
}

impl EloConfig {
    pub const INITIAL: RangeInclusive<i32> = 100..=3000;
    pub const K_MIN: RangeInclusive<i32> = 1..=100;
    pub const K_MAX: RangeInclusive<i32> = 1..=200;
    pub const K_DECAY: RangeInclusive<i32> = 1..=100;

    /// Cada campo na faixa dele, e `k_min` até `k_max`.
    pub fn check(&self) -> Result<(), EloConfigError> {
        within(self.initial, Self::INITIAL, |min, max| {
            EloConfigError::Initial { min, max }
        })?;
        within(self.k_min, Self::K_MIN, |min, max| EloConfigError::KMin {
            min,
            max,
        })?;
        within(self.k_max, Self::K_MAX, |min, max| EloConfigError::KMax {
            min,
            max,
        })?;
        within(self.k_decay, Self::K_DECAY, |min, max| {
            EloConfigError::KDecay { min, max }
        })?;
        if self.k_min > self.k_max {
            return Err(EloConfigError::KOrder);
        }
        Ok(())
    }

    /// O K de quem já tinha `games` jogos. Com uma configuração que não passa
    /// no `check`, ainda é um número (decaimento abaixo de 1 conta como 1).
    pub fn k(&self, games: u32) -> f64 {
        let (min, max) = (f64::from(self.k_min), f64::from(self.k_max));
        let decay = f64::from(self.k_decay.max(1));
        min + (max - min) * (-f64::from(games) / decay).exp()
    }
}

fn within(
    value: i32,
    range: RangeInclusive<i32>,
    error: fn(i32, i32) -> EloConfigError,
) -> Result<(), EloConfigError> {
    if range.contains(&value) {
        Ok(())
    } else {
        Err(error(*range.start(), *range.end()))
    }
}

impl Default for EloConfig {
    fn default() -> Self {
        Self {
            initial: 1500,
            k_min: 32,
            k_max: 80,
            k_decay: 10,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum EloConfigError {
    #[error("rating inicial fora de {min}..={max}")]
    Initial { min: i32, max: i32 },
    #[error("K mínimo fora de {min}..={max}")]
    KMin { min: i32, max: i32 },
    #[error("K máximo fora de {min}..={max}")]
    KMax { min: i32, max: i32 },
    #[error("constante de decaimento fora de {min}..={max}")]
    KDecay { min: i32, max: i32 },
    #[error("K mínimo acima do K máximo")]
    KOrder,
}

/// O sistema Elo com uma configuração já conferida.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Elo {
    config: EloConfig,
}

impl Elo {
    pub fn new(config: EloConfig) -> Result<Self, EloConfigError> {
        config.check()?;
        Ok(Self { config })
    }

    pub const fn config(&self) -> EloConfig {
        self.config
    }

    /// Placar esperado de quem tem `rating` contra quem tem `opponent`, de 0 a 1.
    pub fn expected(rating: i32, opponent: i32) -> f64 {
        expected(f64::from(rating), f64::from(opponent))
    }

    /// Pontos que o vencedor ganha e que o perdedor perde (os dois positivos
    /// ou zero), com os jogos que cada um tinha antes: o duelo, um jogador de
    /// cada lado.
    pub fn points(
        &self,
        winner: i32,
        winner_games: u32,
        loser: i32,
        loser_games: u32,
    ) -> (i32, i32) {
        let (gains, losses) = self.team_points(&[(winner, winner_games)], &[(loser, loser_games)]);
        (gains[0], losses[0])
    }

    /// Pontos que cada jogador do lado vencedor ganha e cada um do perdedor
    /// perde (positivos ou zero), na ordem dada, com o rating e os jogos de
    /// cada um antes deste. Os dois lados com alguém.
    pub fn team_points(
        &self,
        winners: &[(i32, u32)],
        losers: &[(i32, u32)],
    ) -> (Vec<i32>, Vec<i32>) {
        // 1 - E do vencedor é o que ele "não esperava" ganhar, e é também o E
        // do perdedor: a mesma conta para os dois lados dá a soma zero exata
        // quando os K são iguais (`as` satura, e o resultado fica em 0..=K).
        let surprise = 1.0 - expected(mean(winners), mean(losers));
        let points = |&(_, games): &(i32, u32)| (self.config.k(games) * surprise).round() as i32;
        (
            winners.iter().map(points).collect(),
            losers.iter().map(points).collect(),
        )
    }
}

fn expected(rating: f64, opponent: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf((opponent - rating) / 400.0))
}

/// A média aritmética dos ratings de um lado. Com um jogador só, é o rating
/// dele exatamente (a soma de um número é ele, e dividir por 1 não arredonda).
fn mean(side: &[(i32, u32)]) -> f64 {
    let sum: f64 = side.iter().map(|&(rating, _)| f64::from(rating)).sum();
    sum / side.len() as f64
}

impl RatingSystem for Elo {
    type State = i32;

    fn initial(&self) -> i32 {
        self.config.initial
    }

    fn rating(&self, state: &i32) -> i32 {
        *state
    }

    fn game(&self, winners: &[Seat<'_, i32>], losers: &[Seat<'_, i32>]) -> (Vec<i32>, Vec<i32>) {
        let side = |seats: &[Seat<'_, i32>]| -> Vec<(i32, u32)> {
            seats.iter().map(|s| (*s.state, s.games)).collect()
        };
        let (gains, losses) = self.team_points(&side(winners), &side(losers));
        let won = winners.iter().zip(gains);
        let lost = losers.iter().zip(losses);
        (
            won.map(|(s, p)| s.state.saturating_add(p)).collect(),
            lost.map(|(s, p)| s.state.saturating_sub(p)).collect(),
        )
    }

    fn adjust(&self, state: &i32, delta: i32) -> i32 {
        state.saturating_add(delta)
    }
}
