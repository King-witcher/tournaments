//! Formato de slug, o identificador público e legível na URL (tenant,
//! torneio). Cada domínio tem a própria lista de palavras reservadas e os
//! próprios códigos de erro.

const SLUG_LEN: std::ops::RangeInclusive<usize> = 3..=32;

/// a-z, 0-9 e hífen simples entre partes, de 3 a 32 caracteres. Mesmo formato
/// do check das tabelas: `^[a-z0-9]+(-[a-z0-9]+)*$`.
pub fn is_well_formed(slug: &str) -> bool {
    SLUG_LEN.contains(&slug.len())
        && slug.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_format() {
        for ok in ["abc", "quake-br", "qc2v2", "a-b-c", &"a".repeat(32)] {
            assert!(is_well_formed(ok), "{ok}");
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
            assert!(!is_well_formed(bad), "{bad}");
        }
    }
}
