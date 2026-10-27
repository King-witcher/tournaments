//! PATCH de um torneio: junta o que veio com o que está gravado e confere o
//! que o status ainda deixa mudar. A checagem de campeões fica para quem
//! chama, com o pool lido na mesma transação.

use super::{
    setup::Setup,
    status::TournamentStatus,
    texts::{self, Texts},
    types::UpdateTournament,
};
use crate::shared::ApiError;

#[derive(Debug)]
pub struct Edit {
    pub texts: Texts,
    /// `Some` só quando vagas, regra de campeões ou fases mudam de fato.
    pub setup: Option<Setup>,
}

/// Erros: 409 `tournament.finished` (texto depois de encerrar), 409
/// `tournament.setup_locked` (configuração depois de começar) e os de
/// validação dos textos. Mandar a configuração igual à atual não conta como
/// mudança, então o front pode reenviar o formulário inteiro.
pub fn apply(
    status: TournamentStatus,
    current_texts: &Texts,
    current_setup: &Setup,
    patch: UpdateTournament,
) -> Result<Edit, ApiError> {
    let touches_texts =
        patch.name.is_some() || patch.description.is_some() || patch.rules.is_some();
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
    };
    let setup = (setup != *current_setup).then_some(setup);
    if setup.is_some() && !status.setup_editable() {
        return Err(ApiError::conflict("tournament.setup_locked"));
    }
    Ok(Edit { texts, setup })
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
        Setup::from_dto(16, rules(), vec![stage(Bracket::Single, false)])
    }

    fn patch() -> UpdateTournament {
        UpdateTournament {
            name: None,
            description: None,
            rules: None,
            max_participants: None,
            champion_rules: None,
            stages: None,
        }
    }

    #[test]
    fn empty_patch_changes_nothing() {
        let edit = apply(TournamentStatus::Finished, &texts(), &setup(), patch()).unwrap();
        assert_eq!(edit.texts.name, "Copa");
        assert!(edit.setup.is_none());
    }

    #[test]
    fn texts_are_cleaned_and_merged() {
        let p = UpdateTournament {
            name: Some("  Copa 2 ".into()),
            ..patch()
        };
        let edit = apply(TournamentStatus::InProgress, &texts(), &setup(), p).unwrap();
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
        let err = apply(TournamentStatus::Finished, &texts(), &setup(), p).unwrap_err();
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
        let edit = apply(TournamentStatus::InProgress, &texts(), &setup(), p).unwrap();
        assert!(edit.setup.is_none());
    }

    #[test]
    fn setup_locks_once_started() {
        let p = || UpdateTournament {
            stages: Some(vec![stage(Bracket::Double, true)]),
            ..patch()
        };
        let edit = apply(TournamentStatus::CheckIn, &texts(), &setup(), p()).unwrap();
        assert_eq!(edit.setup.unwrap().max_participants, 16);
        let err = apply(TournamentStatus::InProgress, &texts(), &setup(), p()).unwrap_err();
        assert_eq!(err.code, "tournament.setup_locked");
    }
}
