//! Janela de contestação: minutos que o adversário tem para confirmar ou
//! contestar um resultado enviado; sem resposta, o resultado confirma sozinho
//! (domínio `matches`). Muda até o torneio encerrar e vale para os
//! resultados enviados depois da mudança.

use std::ops::RangeInclusive;

use crate::shared::ApiError;

/// Janela quando a criação não diz.
pub const DEFAULT_CONTEST_MINUTES: i32 = 5;
/// Janela aceita, em minutos (o mesmo check está na tabela).
const CONTEST_MINUTES: RangeInclusive<i32> = 1..=1440;

/// 422 `tournament.contest_minutes_invalid` `{min, max}`.
pub fn check(minutes: i32) -> Result<(), ApiError> {
    if CONTEST_MINUTES.contains(&minutes) {
        return Ok(());
    }
    Err(
        ApiError::invalid("tournament.contest_minutes_invalid").with_params([
            ("min", i64::from(*CONTEST_MINUTES.start())),
            ("max", i64::from(*CONTEST_MINUTES.end())),
        ]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_limits() {
        assert!(check(1).is_ok());
        assert!(check(1440).is_ok());
        for bad in [0, -5, 1441] {
            let err = check(bad).unwrap_err();
            assert_eq!(
                (err.code, err.params),
                (
                    "tournament.contest_minutes_invalid",
                    vec![("min", 1), ("max", 1440)]
                )
            );
        }
    }
}
