//! Tokens que aparecem para pessoas (links de convite). A estratégia de ids
//! do projeto está em "Ids" no `docs/decisions.md`.

/// Base58: sem `0`, `O`, `I` e `l` (fáceis de confundir) e sem pontuação,
/// então um duplo clique seleciona o token inteiro.
const ALPHABET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// Caracteres para 128 bits: 58^22 > 2^128.
pub const TOKEN_LEN: usize = 22;

/// Token aleatório de 128 bits em base58, sempre com `TOKEN_LEN` caracteres.
pub fn random_token() -> String {
    let mut bytes = [0u8; 16];
    rand::fill(&mut bytes);
    base58(u128::from_le_bytes(bytes))
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
}
