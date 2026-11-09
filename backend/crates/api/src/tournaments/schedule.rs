//! Data de início e check-in: o que o PATCH muda e a conta da "hora de
//! iniciar". A data é só informação: nada acontece por ela. O check-in se liga
//! ou desliga por torneio e abre só pelo botão da organização, no domínio
//! `lifecycle`.

use chrono::{DateTime, Utc};

use super::status::TournamentStatus;
use crate::shared::ApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Schedule {
    /// Início previsto, em UTC. Opcional em qualquer status, até no passado.
    pub starts_at: Option<DateTime<Utc>>,
    /// Se o torneio tem check-in; sem ele, a inscrição vai direto para em
    /// andamento.
    pub check_in: bool,
}

impl Schedule {
    pub const fn has_check_in(&self) -> bool {
        self.check_in
    }

    /// O início marcado chegou em `now` com o torneio por iniciar (inscrição
    /// ou check-in): ele espera a organização. É o critério de `GET
    /// /me/organizing`, também nas queries dele (`audience`).
    pub fn start_due(&self, status: TournamentStatus, now: DateTime<Utc>) -> bool {
        matches!(
            status,
            TournamentStatus::Registration | TournamentStatus::CheckIn
        ) && self.starts_at.is_some_and(|starts| starts <= now)
    }

    /// O PATCH da data (`Some(None)` limpa) e do check-in. `Some` só quando
    /// muda de fato, então reenviar os valores atuais passa em qualquer
    /// status. Erro: 409 `tournament.schedule_locked` (depois de começar).
    pub fn edit(
        self,
        status: TournamentStatus,
        starts_at: Option<Option<DateTime<Utc>>>,
        check_in: Option<bool>,
    ) -> Result<Option<Self>, ApiError> {
        let next = Self {
            starts_at: starts_at.unwrap_or(self.starts_at),
            check_in: check_in.unwrap_or(self.check_in),
        };
        if next == self {
            return Ok(None);
        }
        if !status.setup_editable() {
            return Err(ApiError::conflict("tournament.schedule_locked"));
        }
        Ok(Some(next))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use TournamentStatus::*;

    fn at(minute: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_800_000_000, 0).unwrap() + Duration::minutes(minute)
    }

    fn schedule() -> Schedule {
        Schedule {
            starts_at: Some(at(60)),
            check_in: true,
        }
    }

    #[test]
    fn start_is_due_from_the_marked_time_until_it_starts() {
        let s = schedule();
        assert!(!s.start_due(CheckIn, at(59)));
        assert!(s.start_due(CheckIn, at(60)));
        assert!(s.start_due(Registration, at(90)));
        for status in [Draft, InProgress, Finished] {
            assert!(!s.start_due(status, at(90)));
        }
        let undated = Schedule {
            starts_at: None,
            ..schedule()
        };
        assert!(!undated.start_due(Registration, at(90)));
    }

    #[test]
    fn edit_changes_only_what_came() {
        let s = schedule();
        let same = s.edit(InProgress, Some(Some(at(60))), Some(true));
        assert_eq!(same.unwrap(), None);
        assert_eq!(
            s.edit(CheckIn, None, Some(false)).unwrap(),
            Some(Schedule {
                check_in: false,
                ..s
            })
        );
        // Tirar a data, ou pôr uma no passado, vale em qualquer status até
        // começar.
        for status in [Draft, Registration, CheckIn] {
            assert_eq!(
                s.edit(status, Some(None), None).unwrap(),
                Some(Schedule {
                    starts_at: None,
                    ..s
                })
            );
            assert_eq!(
                s.edit(status, Some(Some(at(-1000))), None).unwrap(),
                Some(Schedule {
                    starts_at: Some(at(-1000)),
                    ..s
                })
            );
        }
    }

    #[test]
    fn edit_is_locked_once_started() {
        let s = schedule();
        let code = |r: Result<Option<Schedule>, ApiError>| r.unwrap_err().code;
        assert_eq!(
            code(s.edit(InProgress, Some(Some(at(90))), None)),
            "tournament.schedule_locked"
        );
        assert_eq!(
            code(s.edit(Finished, None, Some(false))),
            "tournament.schedule_locked"
        );
    }
}
