//! Screenshot obrigatório (`screenshots_required`): com ele, o jogador só envia
//! um mapa com um screenshot do placar anexado (domínio `screenshots`; a
//! checagem fica no envio, em `matches`). Ligar exige o bucket configurado; o
//! que já está ligado fica gravado e não vale enquanto faltar o bucket (ninguém
//! fica sem poder enviar). Muda até o torneio encerrar, como a janela de
//! contestação, e vale para os envios seguintes.

use super::status::TournamentStatus;
use crate::shared::ApiError;

/// 422: ligar sem o bucket configurado.
const UNAVAILABLE: ApiError = ApiError::invalid("tournament.screenshots_unavailable");

/// Na criação. 422 `tournament.screenshots_unavailable` ligado sem o bucket.
pub fn check_new(required: bool, available: bool) -> Result<(), ApiError> {
    if required && !available {
        return Err(UNAVAILABLE);
    }
    Ok(())
}

/// No PATCH: `Some` só quando muda. 409 `tournament.finished` depois de
/// encerrar e 422 `tournament.screenshots_unavailable` ao ligar sem o bucket
/// (mandar o valor gravado não é mudança, então o front pode reenviar o
/// formulário inteiro).
pub fn edit(
    status: TournamentStatus,
    current: bool,
    patch: Option<bool>,
    available: bool,
) -> Result<Option<bool>, ApiError> {
    let Some(required) = patch.filter(|&v| v != current) else {
        return Ok(None);
    };
    if !status.texts_editable() {
        return Err(ApiError::conflict("tournament.finished"));
    }
    check_new(required, available)?;
    Ok(Some(required))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turning_on_needs_the_bucket() {
        assert!(check_new(false, false).is_ok());
        assert!(check_new(true, true).is_ok());
        let err = check_new(true, false).unwrap_err();
        assert_eq!(err.code, "tournament.screenshots_unavailable");
    }

    #[test]
    fn only_a_change_counts() {
        let s = TournamentStatus::InProgress;
        assert_eq!(edit(s, false, None, true).unwrap(), None);
        assert_eq!(edit(s, true, Some(true), false).unwrap(), None);
        assert_eq!(edit(s, false, Some(true), true).unwrap(), Some(true));
        // Desligar não precisa do bucket.
        assert_eq!(edit(s, true, Some(false), false).unwrap(), Some(false));
        let err = edit(s, false, Some(true), false).unwrap_err();
        assert_eq!(err.code, "tournament.screenshots_unavailable");
    }

    #[test]
    fn locks_when_finished() {
        let s = TournamentStatus::Finished;
        assert_eq!(edit(s, true, Some(true), true).unwrap(), None);
        let err = edit(s, false, Some(true), true).unwrap_err();
        assert_eq!(err.code, "tournament.finished");
    }
}
