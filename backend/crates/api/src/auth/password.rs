//! Senha: tamanho aceito e hash argon2id com os parâmetros padrão do crate (os
//! que o OWASP recomenda). Hash e verificação gastam CPU de propósito, então
//! rodam fora das threads do tokio.

use std::sync::LazyLock;

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};

use crate::shared::ApiError;

const MIN_LEN: usize = 8;
const MAX_LEN: usize = 128;

/// Hash de uma senha que ninguém tem, para a verificação de um username
/// inexistente levar o mesmo tempo e não revelar que ele não existe.
static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| hash_now("senha de ninguém"));

/// Só o tamanho, em caracteres, sem regra de composição (como o NIST 800-63B).
pub fn is_acceptable(password: &str) -> bool {
    (MIN_LEN..=MAX_LEN).contains(&password.chars().count())
}

pub fn validate_password(password: &str) -> Result<(), ApiError> {
    if is_acceptable(password) {
        Ok(())
    } else {
        Err(ApiError::invalid("auth.password_invalid"))
    }
}

/// Hash no formato PHC (`$argon2id$v=19$...`), com salt aleatório.
pub async fn hash_password(password: String) -> Result<String, ApiError> {
    blocking(move || hash_now(&password)).await
}

/// Confere a senha. Sem hash (username inexistente) compara com um hash fixo
/// e devolve `false`.
pub async fn verify_password(hash: Option<String>, password: String) -> Result<bool, ApiError> {
    blocking(move || match hash {
        Some(hash) => verify_now(&hash, &password),
        None => {
            verify_now(&DUMMY_HASH, &password);
            false
        }
    })
    .await
}

fn hash_now(password: &str) -> String {
    Argon2::default()
        .hash_password(password.as_bytes())
        .expect("argon2 com os parâmetros padrão")
        .to_string()
}

fn verify_now(hash: &str, password: &str) -> bool {
    PasswordHash::new(hash).is_ok_and(|parsed| {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    })
}

async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
) -> Result<T, ApiError> {
    tokio::task::spawn_blocking(f).await.map_err(|err| {
        tracing::error!(error = %err, "hash de senha falhou");
        ApiError::internal()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_counts_characters() {
        assert!(is_acceptable("12345678"));
        assert!(!is_acceptable("1234567"));
        assert!(is_acceptable(&"ç".repeat(8)));
        assert!(is_acceptable(&"a".repeat(128)));
        assert!(!is_acceptable(&"a".repeat(129)));
        assert_eq!(
            validate_password("curta").unwrap_err().code,
            "auth.password_invalid"
        );
    }

    #[test]
    fn hash_verifies_only_the_same_password() {
        let hash = hash_now("senha boa 123");
        assert!(hash.starts_with("$argon2id$"));
        assert_ne!(hash, hash_now("senha boa 123"), "salt aleatório");
        assert!(verify_now(&hash, "senha boa 123"));
        assert!(!verify_now(&hash, "senha boa 124"));
        assert!(!verify_now("não é hash", "senha boa 123"));
    }
}
