use crate::shared::ApiError;

const SLUG_LEN: std::ops::RangeInclusive<usize> = 3..=32;

/// Slugs que colidem com rotas do front em `/{locale}/...` (hoje `new` e
/// `invite`) ou que ficam guardados para rotas futuras.
const RESERVED_SLUGS: &[&str] = &[
    "about",
    "admin",
    "api",
    "assets",
    "auth",
    "comunidad",
    "comunidade",
    "comunidades",
    "docs",
    "health",
    "help",
    "invite",
    "login",
    "logout",
    "new",
    "privacy",
    "settings",
    "static",
    "tenants",
    "terms",
    "torneios",
    "torneos",
    "tournaments",
    "users",
];

pub fn validate_slug(slug: &str) -> Result<(), ApiError> {
    // Mesmo formato do check da tabela: ^[a-z0-9]+(-[a-z0-9]+)*$
    let well_formed = SLUG_LEN.contains(&slug.len())
        && slug.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        });
    if !well_formed {
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
    fn slug_format() {
        for ok in ["abc", "quake-br", "qc2v2", "a-b-c", &"a".repeat(32)] {
            assert!(validate_slug(ok).is_ok(), "{ok}");
        }
        for bad in [
            "ab",
            &"a".repeat(33),
            "Quake",
            "quake_br",
            "-abc",
            "abc-",
            "a--b",
            "ação",
            "a b c",
        ] {
            assert_eq!(
                validate_slug(bad).unwrap_err().code,
                "tenant.slug_invalid",
                "{bad}"
            );
        }
        assert_eq!(
            validate_slug("invite").unwrap_err().code,
            "tenant.slug_reserved"
        );
    }
}
