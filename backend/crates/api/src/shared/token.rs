//! Segredos aleatórios e o hash que fica guardado (banco ou Redis): o valor
//! só existe na resposta ou no cookie. A estratégia de ids do projeto está em
//! "Ids" no `docs/decisions.md`.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};

/// Base58: sem `0`, `O`, `I` e `l` (fáceis de confundir) e sem pontuação,
/// então um duplo clique seleciona o token inteiro.
const ALPHABET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// Caracteres para 128 bits: 58^22 > 2^128.
pub const TOKEN_LEN: usize = 22;

/// Token que aparece para pessoas (links de convite): 128 bits em base58,
/// sempre com `TOKEN_LEN` caracteres.
pub fn random_token() -> String {
    let mut bytes = [0u8; 16];
    rand::fill(&mut bytes);
    base58(u128::from_le_bytes(bytes))
}

/// Quantos caracteres do início do token ficam em claro (`token_hint`), só
/// para reconhecer o link numa lista. Sobram ~105 bits de segredo.
pub const TOKEN_HINT_LEN: usize = 4;

/// Os primeiros `TOKEN_HINT_LEN` caracteres do token.
pub fn hint(token: &str) -> &str {
    &token[..TOKEN_HINT_LEN]
}

/// Segredo que só trafega entre máquinas (cookie de sessão, state do OAuth):
/// 256 bits em base64url.
pub fn random_secret() -> String {
    let mut bytes = [0u8; 32];
    rand::fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// SHA-256 do token, que é o que fica guardado.
pub fn hash_token(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

/// O hash em hexadecimal minúsculo, para chave de texto (Redis).
pub fn hex(hash: &[u8]) -> String {
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

/// Comparação em tempo constante, para não vazar o segredo pelo tempo de resposta.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn base58(mut n: u128) -> String {
    let mut out = [ALPHABET[0]; TOKEN_LEN];
    for slot in out.iter_mut().rev() {
        *slot = ALPHABET[(n % 58) as usize];
        n /= 58;
    }
    out.iter().map(|&b| b as char).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base58_has_fixed_width() {
        assert_eq!(base58(0), "1".repeat(TOKEN_LEN));
        assert_eq!(base58(57), format!("{}z", "1".repeat(TOKEN_LEN - 1)));
        assert_eq!(base58(58), format!("{}21", "1".repeat(TOKEN_LEN - 2)));
        assert_eq!(base58(u128::MAX).len(), TOKEN_LEN);
    }

    #[test]
    fn random_tokens_use_only_the_alphabet() {
        for _ in 0..100 {
            let token = random_token();
            assert_eq!(token.len(), TOKEN_LEN);
            assert!(token.bytes().all(|b| ALPHABET.contains(&b)), "{token}");
        }
        assert_ne!(random_token(), random_token());
    }

    #[test]
    fn constant_time_eq_compares_bytes() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
    }
}
