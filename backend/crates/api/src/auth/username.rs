//! Formato do username do login com senha.

use crate::shared::ApiError;

const MIN_LEN: usize = 3;
const MAX_LEN: usize = 20;

/// Username sem os espaços das pontas, se tiver de 3 a 20 letras ou números
/// ASCII, `_`, `.` ou `-`. Só ASCII, para não haver nomes que parecem iguais
/// com letras de outros alfabetos.
pub fn validate_username(raw: &str) -> Result<String, ApiError> {
    let username = raw.trim();
    let well_formed = (MIN_LEN..=MAX_LEN).contains(&username.len())
        && username
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'));
    if well_formed {
        Ok(username.to_owned())
    } else {
        Err(ApiError::invalid("auth.username_invalid"))
    }
}

/// Forma do username que não diferencia maiúsculas, a mesma do índice único
/// (`lower(provider_user_id)`). É a chave do limite de tentativas.
pub fn username_key(username: &str) -> String {
    username.to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_and_trims() {
        assert_eq!(validate_username("  Rapha_Q.1-x ").unwrap(), "Rapha_Q.1-x");
        assert!(validate_username("abc").is_ok());
        assert!(validate_username(&"a".repeat(20)).is_ok());
    }

    #[test]
    fn rejects_bad_usernames() {
        for bad in ["ab", &"a".repeat(21), "com espaço", "joão", "a@b", ""] {
            assert_eq!(
                validate_username(bad).unwrap_err().code,
                "auth.username_invalid",
                "{bad}"
            );
        }
    }

    #[test]
    fn key_ignores_case() {
        assert_eq!(username_key("RaPha"), username_key("rapha"));
    }
}
