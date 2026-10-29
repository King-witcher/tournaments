//! PATCH de um torneio: junta o que veio com o que está gravado e confere o
//! que o status ainda deixa mudar. A checagem de campeões fica para quem
//! chama, com o pool lido na mesma transação.

use super::{
    contest,
    schedule::Schedule,
    setup::Setup,
    status::TournamentStatus,
    texts::{self, Texts},
    types::UpdateTournament,
    veto::VetoSetup,
    visibility::Visibility,
};
use crate::shared::ApiError;

#[derive(Debug)]
pub struct Edit {
    pub texts: Texts,
    /// `Some` quando veio no PATCH.
    pub visibility: Option<Visibility>,
    /// `Some` só quando vagas, regra de campeões ou fases mudam de fato.
    pub setup: Option<Setup>,
    /// `Some` só quando a data de início ou a janela de check-in mudam.
    pub schedule: Option<Schedule>,
    /// `Some` só quando a janela de contestação muda.
    pub contest_minutes: Option<i32>,
    /// `Some` só quando "conta para o rating" muda.
    pub counts_for_rating: Option<bool>,
    /// `Some` quando o status muda junto: desligar o check-in durante o
    /// check-in volta para a inscrição (sem check-in, esse status não existe).
    pub status: Option<TournamentStatus>,
}

/// O torneio como está gravado, que o PATCH muda.
pub struct Current<'a> {
    pub status: TournamentStatus,
    pub texts: &'a Texts,
    pub setup: &'a Setup,
    pub schedule: Schedule,
    pub contest_minutes: i32,
    pub counts_for_rating: bool,
}

/// Erros: 409 `tournament.finished` (texto, visibilidade ou janela de
/// contestação depois de encerrar), 409 `tournament.setup_locked`
/// (configuração, inclusive o veto (`veto`, já resolvido), ou "conta para o
/// rating" depois de começar), os de `Schedule::edit`, o de
/// `contest::check` e os de validação dos textos. Mandar a configuração igual
/// à atual não conta como mudança, então o front pode reenviar o formulário
/// inteiro.
pub fn apply(
    current: Current<'_>,
    veto: Option<VetoSetup>,
    patch: UpdateTournament,
) -> Result<Edit, ApiError> {
    let Current {
        status,
        texts: current_texts,
        setup: current_setup,
        schedule: current_schedule,
        contest_minutes: current_contest_minutes,
        counts_for_rating: current_counts_for_rating,
    } = current;
    let touches_texts = patch.name.is_some()
        || patch.description.is_some()
        || patch.rules.is_some()
        || patch.visibility.is_some();
    if touches_texts && !status.texts_editable() {
        return Err(ApiError::conflict("tournament.finished"));
    }
    let texts = Texts {
        name: match patch.name {
            Some(raw) => texts::name(&raw)?,
            None => current_texts.name.clone(),
        },
        description: match patch.description {
            Some(raw) => texts::description(&raw)?,
            None => current_texts.description.clone(),
        },
        rules: match patch.rules {
            Some(raw) => texts::rules(&raw)?,
            None => current_texts.rules.clone(),
        },
    };

    let setup = Setup {
        max_participants: patch
            .max_participants
            .unwrap_or(current_setup.max_participants),
        champion_rules: patch
            .champion_rules
            .map_or(current_setup.champion_rules, Into::into),
        stages: match patch.stages {
            Some(stages) => stages.into_iter().map(Into::into).collect(),
            None => current_setup.stages.clone(),
        },
        veto: veto.unwrap_or_else(|| current_setup.veto.clone()),
    };
    let setup = (setup != *current_setup).then_some(setup);
    if setup.is_some() && !status.setup_editable() {
        return Err(ApiError::conflict("tournament.setup_locked"));
    }
    let schedule = current_schedule.edit(status, patch.starts_at, patch.check_in_minutes)?;
    let check_in_off = schedule.is_some_and(|s| !s.has_check_in());
    let contest_minutes = patch
        .contest_minutes
        .filter(|&minutes| minutes != current_contest_minutes);
    if let Some(minutes) = contest_minutes {
        if !status.texts_editable() {
            return Err(ApiError::conflict("tournament.finished"));
        }
        contest::check(minutes)?;
    }
    let counts_for_rating = patch
        .counts_for_rating
        .filter(|&counts| counts != current_counts_for_rating);
    if counts_for_rating.is_some() && !status.setup_editable() {
        return Err(ApiError::conflict("tournament.setup_locked"));
    }
    Ok(Edit {
        texts,
        visibility: patch.visibility,
        setup,
        schedule,
        contest_minutes,
        counts_for_rating,
        status: (check_in_off && status == TournamentStatus::CheckIn)
            .then_some(TournamentStatus::Registration),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tournaments::types::{Bracket, ChampionRules, StageFormat};

    fn texts() -> Texts {
        Texts {
            name: "Copa".into(),
            description: "".into(),
            rules: "md3".into(),
        }
    }

    fn stage(bracket: Bracket, reset: bool) -> StageFormat {
        StageFormat::Elimination {
            bracket,
            grand_final_reset: reset,
            best_of: 3,
            finals_best_of: 5,
        }
    }

    fn rules() -> ChampionRules {
        ChampionRules {
            bans_per_game: 1,
            no_repeat: None,
        }
    }

    fn setup() -> Setup {
        Setup::from_dto(
            16,
            rules(),
            vec![stage(Bracket::Single, false)],
            VetoSetup::off(),
        )
    }

    fn schedule() -> Schedule {
        Schedule {
            starts_at: chrono::DateTime::from_timestamp(1_800_000_000, 0),
            check_in_minutes: Some(30),
        }
    }

    fn patch() -> UpdateTournament {
        UpdateTournament {
            name: None,
            description: None,
            rules: None,
            visibility: None,
            max_participants: None,
            champion_rules: None,
            stages: None,
            starts_at: None,
            check_in_minutes: None,
            contest_minutes: None,
            counts_for_rating: None,
            veto: None,
            prints_required: None,
        }
    }

    fn run(status: TournamentStatus, patch: UpdateTournament) -> Result<Edit, ApiError> {
        let current = Current {
            status,
            texts: &texts(),
            setup: &setup(),
            schedule: schedule(),
            contest_minutes: 5,
            counts_for_rating: true,
        };
        apply(current, None, patch)
    }

    #[test]
    fn empty_patch_changes_nothing() {
        let edit = run(TournamentStatus::Finished, patch()).unwrap();
        assert_eq!(edit.texts.name, "Copa");
        assert!(edit.setup.is_none());
        assert!(edit.schedule.is_none());
        assert!(edit.status.is_none());
    }

    #[test]
    fn turning_check_in_off_during_check_in_reopens_registration() {
        let off = || UpdateTournament {
            check_in_minutes: Some(None),
            ..patch()
        };
        let edit = run(TournamentStatus::CheckIn, off()).unwrap();
        assert_eq!(edit.schedule.unwrap().check_in_minutes, None);
        assert_eq!(edit.status, Some(TournamentStatus::Registration));
        let edit = run(TournamentStatus::Registration, off()).unwrap();
        assert_eq!(edit.status, None);
        let err = run(TournamentStatus::InProgress, off()).unwrap_err();
        assert_eq!(err.code, "tournament.schedule_locked");
    }

    #[test]
    fn texts_are_cleaned_and_merged() {
        let p = UpdateTournament {
            name: Some("  Copa 2 ".into()),
            ..patch()
        };
        let edit = run(TournamentStatus::InProgress, p).unwrap();
        assert_eq!(
            (edit.texts.name.as_str(), edit.texts.rules.as_str()),
            ("Copa 2", "md3")
        );
    }

    #[test]
    fn texts_lock_when_finished() {
        let p = UpdateTournament {
            rules: Some("x".into()),
            ..patch()
        };
        let err = run(TournamentStatus::Finished, p).unwrap_err();
        assert_eq!(err.code, "tournament.finished");
    }

    #[test]
    fn visibility_changes_until_finished() {
        let p = || UpdateTournament {
            visibility: Some(Visibility::Private),
            ..patch()
        };
        let edit = run(TournamentStatus::InProgress, p()).unwrap();
        assert_eq!(edit.visibility, Some(Visibility::Private));
        let err = run(TournamentStatus::Finished, p()).unwrap_err();
        assert_eq!(err.code, "tournament.finished");
    }

    #[test]
    fn same_setup_is_not_a_change() {
        // Reset na simples é ignorado, então continua igual ao gravado.
        let p = UpdateTournament {
            max_participants: Some(16),
            champion_rules: Some(rules()),
            stages: Some(vec![stage(Bracket::Single, true)]),
            ..patch()
        };
        let edit = run(TournamentStatus::InProgress, p).unwrap();
        assert!(edit.setup.is_none());
    }

    #[test]
    fn counts_for_rating_locks_once_started() {
        let p = |counts| UpdateTournament {
            counts_for_rating: Some(counts),
            ..patch()
        };
        let edit = run(TournamentStatus::CheckIn, p(false)).unwrap();
        assert_eq!(edit.counts_for_rating, Some(false));
        // O mesmo valor não é mudança, nem depois de começar.
        let edit = run(TournamentStatus::Finished, p(true)).unwrap();
        assert_eq!(edit.counts_for_rating, None);
        let err = run(TournamentStatus::InProgress, p(false)).unwrap_err();
        assert_eq!(err.code, "tournament.setup_locked");
    }

    #[test]
    fn setup_locks_once_started() {
        let p = || UpdateTournament {
            stages: Some(vec![stage(Bracket::Double, true)]),
            ..patch()
        };
        let edit = run(TournamentStatus::CheckIn, p()).unwrap();
        assert_eq!(edit.setup.unwrap().max_participants, 16);
        let err = run(TournamentStatus::InProgress, p()).unwrap_err();
        assert_eq!(err.code, "tournament.setup_locked");
    }
}
