//! Senha: tamanho aceito e hash argon2id com os parâmetros padrão do crate (os
//! que o OWASP recomenda). Hash e verificação gastam CPU e memória de
//! propósito, então rodam fora das threads do tokio, poucos de cada vez.

use std::sync::LazyLock;

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};
use tokio::sync::Semaphore;

use crate::shared::ApiError;

const MIN_LEN: usize = 8;
const MAX_LEN: usize = 128;

/// Hashes e verificações ao mesmo tempo. Cada um aloca 19 MiB (o `m_cost`
/// padrão), e o Render free tem 512 MB: sem fila, logins em massa (até de
/// emails sem conta) derrubariam a API por falta de memória. O resto
/// espera a vez sem thread nem memória.
static HASHING: Semaphore = Semaphore::const_new(2);

/// Hash de uma senha que ninguém tem, para a verificação de um email sem
/// conta levar o mesmo tempo e não revelar que ela não existe.
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

/// Confere a senha. Sem hash (conta inexistente) compara com um hash fixo
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
    let permit = HASHING.acquire().await.map_err(|err| {
        tracing::error!(error = %err, "fila do hash de senha fechada");
        ApiError::internal()
    })?;
    // A vaga vai com o hash: se a requisição cai no meio, ele continua
    // rodando e só a solta no fim.
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        f()
    })
    .await
    .map_err(|err| {
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
