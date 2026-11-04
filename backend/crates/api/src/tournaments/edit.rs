//! PATCH de um torneio: junta o que veio com o que está gravado e confere o
//! que o status ainda deixa mudar. A checagem de campeões fica para quem
//! chama, com o pool lido na mesma transação.

use chrono::{DateTime, Utc};

use super::{
    admission::Admission,
    contest,
    schedule::Schedule,
    setup::Setup,
    slug::validate_slug,
    status::TournamentStatus,
    texts::{self, Texts},
    types::UpdateTournament,
    veto::VetoSetup,
};
use crate::shared::ApiError;

#[derive(Debug)]
pub struct Edit {
    /// `Some` só quando o endereço muda.
    pub slug: Option<String>,
    pub texts: Texts,
    /// `Some` só quando o tipo de inscrição muda.
    pub admission: Option<Admission>,
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
    pub slug: &'a str,
    pub texts: &'a Texts,
    pub admission: Admission,
    pub setup: &'a Setup,
    pub schedule: Schedule,
    pub contest_minutes: i32,
    pub counts_for_rating: bool,
    /// O relógio da requisição, para conferir a data de início nova.
    pub now: DateTime<Utc>,
}

/// Erros: 409 `tournament.slug_locked` (endereço fora do rascunho), 409
/// `tournament.finished` (regras, inscrição ou janela de contestação depois de
/// encerrar; nome e descrição continuam), 409 `tournament.setup_locked`
/// (configuração, inclusive o veto (`veto`, já resolvido), ou "conta para o
/// rating" depois de começar), os de `validate_slug`, os de `Schedule::edit`, o
/// de `contest::check` e os de validação dos textos. Mandar um campo igual ao
/// atual não conta como mudança, então o front pode reenviar o formulário
/// inteiro.
pub fn apply(
    current: Current<'_>,
    veto: Option<VetoSetup>,
    patch: UpdateTournament,
) -> Result<Edit, ApiError> {
    let Current {
        status,
        slug: current_slug,
        texts: current_texts,
        admission: current_admission,
        setup: current_setup,
        schedule: current_schedule,
        contest_minutes: current_contest_minutes,
        counts_for_rating: current_counts_for_rating,
        now,
    } = current;
    let slug = patch.slug.filter(|slug| slug != current_slug);
    if let Some(slug) = &slug {
        if status != TournamentStatus::Draft {
            return Err(ApiError::conflict("tournament.slug_locked"));
        }
        validate_slug(slug)?;
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
    let admission = patch.admission.filter(|a| *a != current_admission);
    if status.is_finished() && (texts.rules != current_texts.rules || admission.is_some()) {
        return Err(ApiError::conflict("tournament.finished"));
    }

    let mode = patch.mode.unwrap_or(current_setup.mode);
    let team_size = match (patch.team_size, patch.mode) {
        (Some(size), _) => size,
        // Trocou de modo sem dizer o tamanho: o padrão do modo novo.
        (None, Some(new)) if new != current_setup.mode => new.default_team_size(),
        (None, _) => current_setup.team_size,
    };
    let setup = Setup {
        mode,
        team_size,
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
        team_rules: patch.team_rules.unwrap_or(current_setup.team_rules),
    };
    let setup = (setup != *current_setup).then_some(setup);
    if setup.is_some() && !status.setup_editable() {
        return Err(ApiError::conflict("tournament.setup_locked"));
    }
    let schedule = current_schedule.edit(status, patch.starts_at, patch.check_in_minutes, now)?;
    let check_in_off = schedule.is_some_and(|s| !s.has_check_in());
    let contest_minutes = patch
        .contest_minutes
        .filter(|&minutes| minutes != current_contest_minutes);
    if let Some(minutes) = contest_minutes {
        if status.is_finished() {
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
        slug,
        texts,
        admission,
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
    use crate::tournaments::{
        setup::TournamentMode,
        types::{Bracket, ChampionRules, StageFormat},
    };

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
            bans_per_series: 1,
            mirror: true,
            same_team: false,
            no_repeat: None,
        }
    }

    fn setup() -> Setup {
        Setup::from_dto(
            TournamentMode::Duel,
            None,
            16,
            rules(),
            vec![stage(Bracket::Single, false)],
            VetoSetup::off(),
            Default::default(),
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
            slug: None,
            mode: None,
            team_size: None,
            name: None,
            description: None,
            rules: None,
            admission: None,
            max_participants: None,
            champion_rules: None,
            stages: None,
            starts_at: None,
            check_in_minutes: None,
            contest_minutes: None,
            counts_for_rating: None,
            veto: None,
            team_rules: None,
            screenshots_required: None,
        }
    }

    fn run(status: TournamentStatus, patch: UpdateTournament) -> Result<Edit, ApiError> {
        let current = Current {
            status,
            slug: "copa",
            texts: &texts(),
            admission: Admission::Open,
            setup: &setup(),
            schedule: schedule(),
            contest_minutes: 5,
            counts_for_rating: true,
            now: DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
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
    fn name_and_description_outlive_the_end_but_rules_do_not() {
        let p = UpdateTournament {
            name: Some(" Copa 2 ".into()),
            description: Some("Agora com descrição".into()),
            // Reenviar as regras de agora não é mudança.
            rules: Some("md3".into()),
            admission: Some(Admission::Open),
            ..patch()
        };
        let edit = run(TournamentStatus::Finished, p).unwrap();
        assert_eq!(edit.texts.name, "Copa 2");
        assert_eq!(edit.texts.description, "Agora com descrição");

        let p = UpdateTournament {
            rules: Some("x".into()),
            ..patch()
        };
        let err = run(TournamentStatus::Finished, p).unwrap_err();
        assert_eq!(err.code, "tournament.finished");
    }

    #[test]
    fn admission_changes_until_finished() {
        let p = || UpdateTournament {
            admission: Some(Admission::InviteOnly),
            ..patch()
        };
        let edit = run(TournamentStatus::InProgress, p()).unwrap();
        assert_eq!(edit.admission, Some(Admission::InviteOnly));
        let err = run(TournamentStatus::Finished, p()).unwrap_err();
        assert_eq!(err.code, "tournament.finished");
    }

    #[test]
    fn the_address_changes_only_in_a_draft() {
        let p = |slug: &str| UpdateTournament {
            slug: Some(slug.into()),
            ..patch()
        };
        let edit = run(TournamentStatus::Draft, p("liga")).unwrap();
        assert_eq!(edit.slug.as_deref(), Some("liga"));
        // A validação da criação.
        let err = run(TournamentStatus::Draft, p("new")).unwrap_err();
        assert_eq!(err.code, "tournament.slug_reserved");
        let err = run(TournamentStatus::Draft, p("X")).unwrap_err();
        assert_eq!(err.code, "tournament.slug_invalid");
        // O mesmo não é mudança; o novo, depois de publicar, é recusado.
        use TournamentStatus::{CheckIn, Finished, InProgress, Registration};
        for status in [Registration, CheckIn, InProgress, Finished] {
            assert_eq!(run(status, p("copa")).unwrap().slug, None);
            let err = run(status, p("liga")).unwrap_err();
            assert_eq!(err.code, "tournament.slug_locked");
        }
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
    fn switching_mode_takes_its_default_team_size() {
        let p = |mode, team_size| UpdateTournament {
            mode: Some(mode),
            team_size,
            ..patch()
        };
        let setup = run(TournamentStatus::Registration, p(TournamentMode::Tdm, None))
            .unwrap()
            .setup
            .unwrap();
        assert_eq!((setup.mode, setup.team_size), (TournamentMode::Tdm, 2));
        let setup = run(
            TournamentStatus::Registration,
            p(TournamentMode::Tdm, Some(4)),
        )
        .unwrap()
        .setup
        .unwrap();
        assert_eq!(setup.team_size, 4);
        // O mesmo modo não é mudança; depois de começar, o novo é recusado.
        let edit = run(TournamentStatus::InProgress, p(TournamentMode::Duel, None)).unwrap();
        assert!(edit.setup.is_none());
        let err = run(TournamentStatus::InProgress, p(TournamentMode::Tdm, None)).unwrap_err();
        assert_eq!(err.code, "tournament.setup_locked");
    }

    #[test]
    fn who_acts_for_the_team_locks_once_started() {
        use crate::tournaments::team_rules::{TeamActor, TeamRules};
        let p = |report| UpdateTournament {
            team_rules: Some(TeamRules {
                report,
                ..TeamRules::EVERYONE
            }),
            ..patch()
        };
        let edit = run(TournamentStatus::CheckIn, p(TeamActor::Captain)).unwrap();
        assert_eq!(edit.setup.unwrap().team_rules.report, TeamActor::Captain);
        // O mesmo valor não é mudança; a de resultado também trava.
        let edit = run(TournamentStatus::InProgress, p(TeamActor::Anyone)).unwrap();
        assert!(edit.setup.is_none());
        let err = run(TournamentStatus::InProgress, p(TeamActor::Captain)).unwrap_err();
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
