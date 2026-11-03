use crate::shared::{ApiError, slug::is_well_formed};

/// Slugs que colidem com rotas do front em `/{locale}/...` (hoje `new`,
/// `invite`, `login`, `signup`, `account`, `email` e `veto`) ou que ficam
/// guardados para rotas futuras.
const RESERVED_SLUGS: &[&str] = &[
    "about",
    "account",
    "admin",
    "api",
    "assets",
    "auth",
    "comunidad",
    "comunidade",
    "comunidades",
    "docs",
    "email",
    "health",
    "help",
    "invite",
    "login",
    "logout",
    "new",
    "privacy",
    "settings",
    "signup",
    "static",
    "tenants",
    "terms",
    "torneios",
    "torneos",
    "tournaments",
    "users",
    "veto",
];

/// Na criação e na troca de endereço.
pub fn validate_slug(slug: &str) -> Result<(), ApiError> {
    if !is_well_formed(slug) {
        return Err(ApiError::invalid("tenant.slug_invalid"));
    }
    if RESERVED_SLUGS.contains(&slug) {
        return Err(ApiError::invalid("tenant.slug_reserved"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_codes() {
        assert!(validate_slug("quake-br").is_ok());
        assert_eq!(
            validate_slug("Quake").unwrap_err().code,
            "tenant.slug_invalid"
        );
        for reserved in ["invite", "veto"] {
            assert_eq!(
                validate_slug(reserved).unwrap_err().code,
                "tenant.slug_reserved"
            );
        }
    }
}
