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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_text_trims_and_limits() {
        assert_eq!(clean_text("  Quake BR ", 64, "x").unwrap(), "Quake BR");
        assert_eq!(clean_text(&"é".repeat(64), 64, "x").unwrap().len(), 128);
        for bad in ["", "   ", "a\nb", &"é".repeat(65)] {
            assert_eq!(clean_text(bad, 64, "x").unwrap_err().code, "x", "{bad:?}");
        }
    }
}
