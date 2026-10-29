//! A configuração do Elo da comunidade: o DTO, a conversão para o engine e
//! um código por erro de faixa (as faixas são do engine).

use engine::rating::{EloConfig, EloConfigError};

use super::types::RatingSettings;
use crate::shared::ApiError;

impl From<EloConfig> for RatingSettings {
    fn from(c: EloConfig) -> Self {
        Self {
            initial: c.initial,
            k_min: c.k_min,
            k_max: c.k_max,
            k_decay: c.k_decay,
        }
    }
}

/// 422 `rating.initial_invalid`, `rating.k_min_invalid`,
/// `rating.k_max_invalid`, `rating.k_decay_invalid` (com `min` e `max`) ou
/// `rating.k_order_invalid` (K mínimo acima do máximo).
pub fn check(s: &RatingSettings) -> Result<EloConfig, ApiError> {
    let config = EloConfig {
        initial: s.initial,
        k_min: s.k_min,
        k_max: s.k_max,
        k_decay: s.k_decay,
    };
    config.check().map_err(error)?;
    Ok(config)
}

fn error(e: EloConfigError) -> ApiError {
    let range = |code, min: i32, max: i32| {
        ApiError::invalid(code).with_params([("min", i64::from(min)), ("max", i64::from(max))])
    };
    match e {
        EloConfigError::Initial { min, max } => range("rating.initial_invalid", min, max),
        EloConfigError::KMin { min, max } => range("rating.k_min_invalid", min, max),
        EloConfigError::KMax { min, max } => range("rating.k_max_invalid", min, max),
        EloConfigError::KDecay { min, max } => range("rating.k_decay_invalid", min, max),
        EloConfigError::KOrder => ApiError::invalid("rating.k_order_invalid"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> RatingSettings {
        EloConfig::default().into()
    }

    #[test]
    fn codes() {
        assert_eq!(check(&settings()).unwrap(), EloConfig::default());
        let cases = [
            (
                RatingSettings {
                    initial: 50,
                    ..settings()
                },
                ("rating.initial_invalid", vec![("min", 100), ("max", 3000)]),
            ),
            (
                RatingSettings {
                    k_decay: 0,
                    ..settings()
                },
                ("rating.k_decay_invalid", vec![("min", 1), ("max", 100)]),
            ),
            (
                RatingSettings {
                    k_min: 90,
                    ..settings()
                },
                ("rating.k_order_invalid", vec![]),
            ),
        ];
        for (input, want) in cases {
            let err = check(&input).unwrap_err();
            assert_eq!((err.code, err.params), want);
        }
    }
}
