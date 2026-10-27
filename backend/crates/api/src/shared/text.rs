//! Validação de texto livre que vem do usuário (nomes, apelidos).

use super::ApiError;

/// Texto de uma linha: sem espaços nas pontas, sem caracteres de controle,
/// de 1 a `max` caracteres. Fora disso, 422 com `code`.
pub fn clean_text(raw: &str, max: usize, code: &'static str) -> Result<String, ApiError> {
    let text = raw.trim();
    let len = text.chars().count();
    if len == 0 || len > max || text.chars().any(char::is_control) {
        return Err(ApiError::invalid(code));
    }
    Ok(text.to_owned())
}

/// Texto longo de várias linhas (Markdown de descrição e regras): quebras de
/// linha viram `\n`, sem espaços nas pontas, sem outros caracteres de controle
/// além de tabulação, de 0 a `max` caracteres. Fora disso, 422 com `code` e
/// `max` nos parâmetros.
pub fn clean_long_text(raw: &str, max: usize, code: &'static str) -> Result<String, ApiError> {
    let text = raw.replace("\r\n", "\n").replace('\r', "\n");
    let text = text.trim();
    let allowed = |c: char| !c.is_control() || c == '\n' || c == '\t';
    if text.chars().count() > max || !text.chars().all(allowed) {
        return Err(ApiError::invalid(code).with_params([("max", max as i64)]));
    }
    Ok(text.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_long_text_keeps_lines() {
        assert_eq!(
            clean_long_text("\r\n# Regras\r\n\r\n- md3\t\n", 100, "x").unwrap(),
            "# Regras\n\n- md3"
        );
        assert_eq!(clean_long_text("   ", 100, "x").unwrap(), "");
        for bad in ["a\u{0}b", "a\u{7}b", &"é".repeat(101)] {
            let err = clean_long_text(bad, 100, "x").unwrap_err();
            assert_eq!((err.code, err.params), ("x", vec![("max", 100)]), "{bad:?}");
        }
    }

    #[test]
    fn clean_text_trims_and_limits() {
        assert_eq!(clean_text("  Quake BR ", 64, "x").unwrap(), "Quake BR");
        assert_eq!(clean_text(&"é".repeat(64), 64, "x").unwrap().len(), 128);
        for bad in ["", "   ", "a\nb", &"é".repeat(65)] {
            assert_eq!(clean_text(bad, 64, "x").unwrap_err().code, "x", "{bad:?}");
        }
    }
}
