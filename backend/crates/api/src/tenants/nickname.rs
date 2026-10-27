//! Apelido do membro em um tenant (`tenant_members.nickname`).

use crate::shared::{ApiError, text::clean_text};

const NICKNAME_MAX: usize = 32;

/// Apelido que o próprio membro escolheu: 1 a 32 caracteres, sem espaços nas pontas.
pub fn validate_nickname(raw: &str) -> Result<String, ApiError> {
    clean_text(raw, NICKNAME_MAX, "member.nickname_invalid")
}

/// Apelido inicial de quem entra num tenant: o nome do usuário, cortado no limite.
pub fn default_nickname(display_name: &str) -> String {
    let cut: String = display_name
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .trim()
        .chars()
        .take(NICKNAME_MAX)
        .collect();
    match cut.trim_end() {
        "" => "player".to_owned(),
        name => name.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_nickname_fits_the_column() {
        assert_eq!(default_nickname("  Quaker "), "Quaker");
        assert_eq!(default_nickname(&"á".repeat(40)).chars().count(), 32);
        assert_eq!(
            default_nickname(&format!("{} x", "a".repeat(31))),
            "a".repeat(31)
        );
        assert_eq!(default_nickname(" \n "), "player");
    }
}
