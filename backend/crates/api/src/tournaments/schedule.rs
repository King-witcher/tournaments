//! Data de início e janela de check-in: limites, o que o PATCH muda e a
//! abertura automática do check-in, que `db::find` e `db::list` aplicam ao
//! ler (não há job em segundo plano). As transições manuais ficam no domínio
//! `lifecycle`.

use std::ops::RangeInclusive;

use chrono::{DateTime, Duration, Utc};

use super::status::TournamentStatus;
use crate::shared::ApiError;

/// Janela de check-in aceita, em minutos (o mesmo check está na tabela).
const CHECK_IN_MINUTES: RangeInclusive<i32> = 5..=1440;
/// Janela quando a criação não diz.
pub const DEFAULT_CHECK_IN_MINUTES: i32 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Schedule {
    /// Início previsto, em UTC. Só o rascunho fica sem.
    pub starts_at: Option<DateTime<Utc>>,
    /// Minutos antes do início em que o check-in abre; `None`: sem check-in.
    pub check_in_minutes: Option<i32>,
}

impl Schedule {
    /// 422 `tournament.check_in_minutes_invalid` `{min, max}`.
    pub fn check(&self) -> Result<(), ApiError> {
        match self.check_in_minutes {
            Some(minutes) if !CHECK_IN_MINUTES.contains(&minutes) => Err(ApiError::invalid(
                "tournament.check_in_minutes_invalid",
            )
            .with_params([
                ("min", i64::from(*CHECK_IN_MINUTES.start())),
                ("max", i64::from(*CHECK_IN_MINUTES.end())),
            ])),
            _ => Ok(()),
        }
    }

    pub const fn has_check_in(&self) -> bool {
        self.check_in_minutes.is_some()
    }

    /// Quando o check-in abre sozinho; `None` sem data ou sem check-in.
    pub fn check_in_opens_at(&self) -> Option<DateTime<Utc>> {
        Some(self.starts_at? - Duration::minutes(i64::from(self.check_in_minutes?)))
    }

    /// Abertura automática: a inscrição vira check-in quando a janela abre,
    /// a menos que o organizador tenha reaberto a inscrição depois dessa
    /// abertura (`reopened_at`). Mudar a data ou a janela para uma abertura
    /// posterior à reabertura reativa. Só anda para a frente: adiar o início
    /// com o check-in aberto não o fecha (o organizador reabre a inscrição).
    pub fn check_in_due(
        &self,
        status: TournamentStatus,
        reopened_at: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> bool {
        status == TournamentStatus::Registration
            && self.check_in_opens_at().is_some_and(|opens| {
                opens <= now && reopened_at.is_none_or(|reopened| reopened < opens)
            })
    }

    /// O PATCH da data e da janela (`Some(None)` limpa). `Some` só quando
    /// muda de fato, então reenviar os valores atuais passa em qualquer
    /// status. Erros: 409 `tournament.schedule_locked` (depois de começar),
    /// 422 `tournament.starts_at_required` (sem data fora do rascunho) e os
    /// de `check`.
    pub fn edit(
        self,
        status: TournamentStatus,
        starts_at: Option<Option<DateTime<Utc>>>,
        check_in_minutes: Option<Option<i32>>,
    ) -> Result<Option<Self>, ApiError> {
        let next = Self {
            starts_at: starts_at.unwrap_or(self.starts_at),
            check_in_minutes: check_in_minutes.unwrap_or(self.check_in_minutes),
        };
        if next == self {
            return Ok(None);
        }
        if !status.setup_editable() {
            return Err(ApiError::conflict("tournament.schedule_locked"));
        }
        if next.starts_at.is_none() && status != TournamentStatus::Draft {
            return Err(ApiError::invalid("tournament.starts_at_required"));
        }
        next.check()?;
        Ok(Some(next))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use TournamentStatus::*;

    fn at(minute: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_800_000_000, 0).unwrap() + Duration::minutes(minute)
    }

    /// Começa no minuto 60, check-in de 30: abre no minuto 30.
    fn schedule() -> Schedule {
        Schedule {
            starts_at: Some(at(60)),
            check_in_minutes: Some(30),
        }
    }

    #[test]
    fn check_in_opens_with_the_window() {
        let s = schedule();
        assert_eq!(s.check_in_opens_at(), Some(at(30)));
        assert!(!s.check_in_due(Registration, None, at(29)));
        assert!(s.check_in_due(Registration, None, at(30)));
        assert!(s.check_in_due(Registration, None, at(90)));
        for status in [Draft, CheckIn, InProgress, Finished] {
            assert!(!s.check_in_due(status, None, at(45)));
        }
    }

    #[test]
    fn no_check_in_or_no_date_never_opens() {
        let off = Schedule {
            check_in_minutes: None,
            ..schedule()
        };
        let undated = Schedule {
            starts_at: None,
            ..schedule()
        };
        for s in [off, undated] {
            assert_eq!(s.check_in_opens_at(), None);
            assert!(!s.check_in_due(Registration, None, at(90)));
        }
    }

    #[test]
    fn reopening_suspends_until_the_opening_moves_past_it() {
        // Reaberta no minuto 40, com a janela aberta desde o 30.
        let s = schedule();
        assert!(!s.check_in_due(Registration, Some(at(40)), at(50)));
        // Reaberta antes de a janela abrir: a automática continua valendo.
        assert!(s.check_in_due(Registration, Some(at(20)), at(30)));
        // Adiar o início para o minuto 90 abre no 60, depois da reabertura.
        let later = Schedule {
            starts_at: Some(at(90)),
            ..s
        };
        assert!(!later.check_in_due(Registration, Some(at(40)), at(59)));
        assert!(later.check_in_due(Registration, Some(at(40)), at(60)));
    }

    #[test]
    fn edit_changes_only_what_came() {
        let s = schedule();
        let same = s.edit(InProgress, Some(Some(at(60))), Some(Some(30)));
        assert_eq!(same.unwrap(), None);
        assert_eq!(
            s.edit(CheckIn, None, Some(None)).unwrap(),
            Some(Schedule {
                check_in_minutes: None,
                ..s
            })
        );
        assert_eq!(
            s.edit(Draft, Some(None), None).unwrap(),
            Some(Schedule {
                starts_at: None,
                ..s
            })
        );
    }

    #[test]
    fn edit_errors() {
        let s = schedule();
        let code = |r: Result<Option<Schedule>, ApiError>| r.unwrap_err().code;
        assert_eq!(
            code(s.edit(InProgress, Some(Some(at(90))), None)),
            "tournament.schedule_locked"
        );
        assert_eq!(
            code(s.edit(Finished, None, Some(None))),
            "tournament.schedule_locked"
        );
        assert_eq!(
            code(s.edit(Registration, Some(None), None)),
            "tournament.starts_at_required"
        );
        let err = s.edit(Draft, None, Some(Some(4))).unwrap_err();
        assert_eq!(
            (err.code, err.params),
            (
                "tournament.check_in_minutes_invalid",
                vec![("min", 5), ("max", 1440)]
            )
        );
        assert!(s.edit(Draft, None, Some(Some(1440))).is_ok());
    }
}
