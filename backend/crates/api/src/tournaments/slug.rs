use crate::shared::{ApiError, slug::is_well_formed};

/// Slugs que colidem com rotas do front em `/{locale}/{tenant}/...` (hoje
/// `new`) ou que ficam guardados para rotas futuras do tenant.
const RESERVED_SLUGS: &[&str] = &[
    "admin",
    "ajustes",
    "configuracoes",
    "edit",
    "invites",
    "members",
    "membros",
    "miembros",
    "new",
    "novo",
    "nuevo",
    "ranking",
    "rankings",
    "rating",
    "settings",
    "stats",
    "torneios",
    "torneos",
    "tournaments",
];

pub fn validate_slug(slug: &str) -> Result<(), ApiError> {
    if !is_well_formed(slug) {
        return Err(ApiError::invalid("tournament.slug_invalid"));
    }
    if RESERVED_SLUGS.contains(&slug) {
        return Err(ApiError::invalid("tournament.slug_reserved"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_codes() {
        assert!(validate_slug("copa-de-outubro").is_ok());
        assert_eq!(
            validate_slug("copa_outubro").unwrap_err().code,
            "tournament.slug_invalid"
        );
        assert_eq!(
            validate_slug("new").unwrap_err().code,
            "tournament.slug_reserved"
        );
    }
}
