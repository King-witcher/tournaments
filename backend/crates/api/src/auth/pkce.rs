//! O formato do que o navegador gera para o OAuth com PKCE (RFC 7636): o
//! state, o code verifier e o challenge dele. O front confere o state, e o
//! provedor, o verifier contra o challenge. Para entrar, a API não guarda
//! nenhum dos três; para vincular, guarda o challenge com a sessão e confere
//! o verifier contra ele (`challenge_of`). Aqui também se recusa o que o
//! front não geraria.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};

/// Caracteres "unreserved" da RFC 3986, os do verifier.
fn unreserved(s: &str) -> bool {
    s.bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~'))
}

/// State de 22 (128 bits em base64url) a 128 caracteres "unreserved".
pub fn is_state(s: &str) -> bool {
    (22..=128).contains(&s.len()) && unreserved(s)
}

/// Verifier de 43 a 128 caracteres "unreserved" (RFC 7636, 4.1).
pub fn is_verifier(s: &str) -> bool {
    (43..=128).contains(&s.len()) && unreserved(s)
}

/// Challenge S256: o SHA-256 do verifier em base64url sem `=`, 43 caracteres.
pub fn is_challenge(s: &str) -> bool {
    s.len() == 43
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}

/// O challenge S256 de `verifier`, como o navegador calcula.
pub fn challenge_of(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O exemplo do apêndice B da RFC 7636.
    const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    const CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

    #[test]
    fn accepts_what_the_browser_generates() {
        assert_eq!(challenge_of(VERIFIER), CHALLENGE);
        assert!(is_verifier(VERIFIER));
        assert!(is_challenge(CHALLENGE));
        assert!(is_state(VERIFIER));
        assert!(
            is_verifier(&"a.b~c_d-".repeat(16)),
            "128, com os quatro sinais"
        );
    }

    #[test]
    fn refuses_other_lengths_and_characters() {
        assert!(!is_verifier(&VERIFIER[..42]));
        assert!(!is_verifier(&"a".repeat(129)));
        assert!(!is_verifier(&format!("{}=", &VERIFIER[..42])));
        assert!(!is_challenge(&CHALLENGE[..42]));
        assert!(!is_challenge(&format!("{}=", &CHALLENGE[..42])));
        assert!(
            !is_challenge(&format!("{}.", &CHALLENGE[..42])),
            "base64url não tem ponto"
        );
        assert!(!is_state(&"a".repeat(21)));
        assert!(!is_state(&"a".repeat(129)));
        assert!(!is_state(&format!("{}&x=1", "a".repeat(22))));
        assert!(!is_state(&"ã".repeat(22)));
    }
}
