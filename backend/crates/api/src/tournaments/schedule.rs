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
    /// Início previsto, em UTC. Opcional em qualquer status: sem data o check-in não abre sozinho
    /// (o organizador abre) e não há aviso de hora de iniciar.
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

    /// A data de início deixa tempo para a inscrição e o check-in: está no
    /// futuro e a janela de check-in (contada a partir dela) ainda não abriu
    /// em `now`. Sem data, nada a conferir. Erros: 422
    /// `tournament.starts_at_past` e `tournament.starts_at_too_soon` com
    /// `minutes` (a janela, que é a antecedência mínima).
    pub fn check_start(&self, now: DateTime<Utc>) -> Result<(), ApiError> {
        let Some(starts_at) = self.starts_at else {
            return Ok(());
        };
        if starts_at <= now {
            return Err(ApiError::invalid("tournament.starts_at_past"));
        }
        match self.check_in_minutes {
            Some(minutes) if starts_at - Duration::minutes(i64::from(minutes)) <= now => {
                Err(ApiError::invalid("tournament.starts_at_too_soon")
                    .with_params([("minutes", i64::from(minutes))]))
            }
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

    /// O início marcado chegou em `now` com o torneio por iniciar (inscrição
    /// ou check-in): ele espera a organização. É o critério do aviso
    /// `tournament.start_due` (que sai uma vez) e de `GET /me/organizing`,
    /// também nas queries deles (`starts_to_remind`, `audience`).
    pub fn start_due(&self, status: TournamentStatus, now: DateTime<Utc>) -> bool {
        matches!(
            status,
            TournamentStatus::Registration | TournamentStatus::CheckIn
        ) && self.starts_at.is_some_and(|starts| starts <= now)
    }

    /// O PATCH da data e da janela (`Some(None)` limpa). `Some` só quando
    /// muda de fato, então reenviar os valores atuais passa em qualquer
    /// status. A data só é conferida (`check_start` em `now`) quando muda:
    /// editar outra coisa de um torneio cuja data já passou continua
    /// permitido. Erros: 409 `tournament.schedule_locked` (depois de começar)
    /// e os de `check` e `check_start`.
    pub fn edit(
        self,
        status: TournamentStatus,
        starts_at: Option<Option<DateTime<Utc>>>,
        check_in_minutes: Option<Option<i32>>,
        now: DateTime<Utc>,
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
        next.check()?;
        if next.starts_at != self.starts_at {
            next.check_start(now)?;
        }
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
        let now = at(-1000);
        let same = s.edit(InProgress, Some(Some(at(60))), Some(Some(30)), now);
        assert_eq!(same.unwrap(), None);
        assert_eq!(
            s.edit(CheckIn, None, Some(None), now).unwrap(),
            Some(Schedule {
                check_in_minutes: None,
                ..s
            })
        );
        // Tirar a data vale em qualquer status até começar.
        for status in [Draft, Registration, CheckIn] {
            assert_eq!(
                s.edit(status, Some(None), None, now).unwrap(),
                Some(Schedule {
                    starts_at: None,
                    ..s
                })
            );
        }
    }

    #[test]
    fn edit_errors() {
        let s = schedule();
        let now = at(-1000);
        let code = |r: Result<Option<Schedule>, ApiError>| r.unwrap_err().code;
        assert_eq!(
            code(s.edit(InProgress, Some(Some(at(90))), None, now)),
            "tournament.schedule_locked"
        );
        assert_eq!(
            code(s.edit(Finished, None, Some(None), now)),
            "tournament.schedule_locked"
        );
        let err = s.edit(Draft, None, Some(Some(4)), now).unwrap_err();
        assert_eq!(
            (err.code, err.params),
            (
                "tournament.check_in_minutes_invalid",
                vec![("min", 5), ("max", 1440)]
            )
        );
        assert!(s.edit(Draft, None, Some(Some(1440)), now).is_ok());
    }

    #[test]
    fn the_start_needs_the_check_in_window_ahead() {
        let s = schedule();
        let err = |now| s.check_start(now).unwrap_err();
        // Começa no minuto 60, janela de 30: o limite é o minuto 30.
        assert!(s.check_start(at(29)).is_ok());
        let soon = err(at(30));
        assert_eq!(
            (soon.code, soon.params),
            ("tournament.starts_at_too_soon", vec![("minutes", 30)])
        );
        assert_eq!(err(at(60)).code, "tournament.starts_at_past");
        assert_eq!(err(at(90)).code, "tournament.starts_at_past");
        // Sem check-in, só não pode ser passada; sem data, nada a conferir.
        let off = Schedule {
            check_in_minutes: None,
            ..s
        };
        assert!(off.check_start(at(59)).is_ok());
        assert_eq!(
            off.check_start(at(60)).unwrap_err().code,
            "tournament.starts_at_past"
        );
        let undated = Schedule {
            starts_at: None,
            ..s
        };
        assert!(undated.check_start(at(90)).is_ok());
    }

    #[test]
    fn edit_checks_the_start_only_when_it_changes() {
        let s = schedule();
        let now = at(45);
        // Mesma data (já dentro da janela): passa, mesmo mudando a janela.
        let kept = s.edit(Registration, Some(Some(at(60))), Some(Some(15)), now);
        assert!(kept.is_ok());
        // Mover a data para dentro da janela ou para o passado: recusa.
        let code = |start| {
            s.edit(Registration, Some(Some(start)), None, now)
                .unwrap_err()
                .code
        };
        assert_eq!(code(at(70)), "tournament.starts_at_too_soon");
        assert_eq!(code(at(40)), "tournament.starts_at_past");
        assert!(s.edit(Registration, Some(Some(at(80))), None, now).is_ok());
    }
}
