//! O pedido que se confirma com o que vier primeiro do mesmo email: o link
//! aberto ou o código digitado. O cadastro (`signups.rs`), entrar por email, a
//! troca de email e a prova do email de agora (`links.rs`) dividem isto. Cada um
//! tem sua família de chaves e, por no máximo o TTL do pedido:
//! - `{registro}:{dono}`: hash com `token` (hash do token do link, em
//!   hexadecimal), `code` (hash do código, com o do token como sal),
//!   `code_attempts` (códigos conferidos) e os campos do pedido;
//! - `{ponteiro}:{hash do token}`: o dono. Vale uma vez só (`GETDEL`), pelo link
//!   ou pelo código.
//!
//! E, somando todas as famílias: `email_code_failures:{dono}`, os códigos
//! conferidos desde o último acerto, que somem 24 h depois do último erro.
//!
//! Um pedido por dono: pedir de novo derruba o anterior (o link e o código
//! velhos param de valer) e zera `code_attempts`, mas não a soma.

use std::collections::HashMap;

use redis::{Cmd, RedisResult};

use crate::shared::{Redis, token};

/// Códigos errados que um pedido aceita antes de o código morrer (o link
/// continua valendo; pedir de novo manda outro código).
const CODE_ATTEMPTS: u32 = 5;

/// Códigos conferidos por dono, somando os pedidos: quatro pedidos inteiros de
/// erros. Quem digita errado não chega perto; com um milhão de códigos, quem
/// tenta adivinhar acerta com no máximo 0,002% de chance a cada 24 h. Passou
/// disso, o código não vale, e o link continua: dá para travar o código de
/// alguém, mas não o login dele.
const CODE_FAILURES: u32 = 20;

const CODE_FAILURES_TTL_SECS: i64 = 24 * 60 * 60;

/// Os prefixos das duas chaves de um tipo de pedido.
#[derive(Clone, Copy)]
pub(super) struct Family {
    pub record: &'static str,
    pub pointer: &'static str,
}

/// O que vai no email: o token do link e o código.
pub struct Secrets {
    pub link: String,
    pub code: String,
}

impl Family {
    fn record_key(self, redis: &Redis, owner: &str) -> String {
        redis.key(format_args!("{}:{owner}", self.record))
    }

    fn pointer_key(self, redis: &Redis, hash: &str) -> String {
        redis.key(format_args!("{}:{hash}", self.pointer))
    }

    /// Guarda o pedido de `owner` no lugar do anterior, com os campos `fields`.
    pub async fn store(
        self,
        redis: &Redis,
        owner: &str,
        fields: &[(&str, &str)],
        ttl_secs: u64,
    ) -> RedisResult<Secrets> {
        let key = self.record_key(redis, owner);
        let old: Option<String> = redis.query(&Cmd::hget(&key, "token")).await?;

        let link = token::random_token();
        let code = token::random_code();
        let hash = token::hex(&token::hash_token(&link));
        let salted = code_hash(&hash, &code);
        let mut all = vec![("token", hash.as_str()), ("code", salted.as_str())];
        all.extend_from_slice(fields);

        let mut pipe = redis::pipe();
        pipe.atomic();
        if let Some(old) = old {
            pipe.del(self.pointer_key(redis, &old)).ignore();
        }
        pipe.del(&key)
            .ignore()
            .hset_multiple(&key, &all)
            .ignore()
            .expire(&key, ttl_secs as i64)
            .ignore()
            .set_ex(self.pointer_key(redis, &hash), owner, ttl_secs)
            .ignore();
        redis.query_pipe::<()>(&pipe).await?;
        Ok(Secrets { link, code })
    }

    /// Dono e campos do pedido deste link, que sai do Redis; `None` se o link
    /// expirou, já foi usado ou foi trocado por um mais novo.
    pub async fn take_by_link(
        self,
        redis: &Redis,
        link: &str,
    ) -> RedisResult<Option<(String, HashMap<String, String>)>> {
        self.take(redis, &token::hex(&token::hash_token(link)))
            .await
    }

    /// Os campos do pedido de `owner`, que sai do Redis, se o código confere;
    /// `None` em qualquer recusa, sem dizer qual: errado, sem pedido, código
    /// morto, teto dos erros do dono. Cada conferência conta antes de comparar,
    /// em comandos que pedidos ao mesmo tempo não leem iguais (`HINCRBY`, `INCR`):
    /// o mesmo pedido nunca compara mais que `CODE_ATTEMPTS`, nem o mesmo dono
    /// mais que `CODE_FAILURES`. São sempre as mesmas três idas ao Redis, para o
    /// tempo não dizer se há pedido: as rotas que não podem revelar quem tem conta
    /// dependem disso (só a conferência certa faz mais).
    pub async fn check_code(
        self,
        redis: &Redis,
        owner: &str,
        code: &str,
    ) -> RedisResult<Option<HashMap<String, String>>> {
        let key = self.record_key(redis, owner);
        let failures = failures_key(redis, owner);
        let (stored, link_hash): (Option<String>, Option<String>) = redis
            .query(redis::cmd("HMGET").arg(&key).arg("code").arg("token"))
            .await?;
        let (Some(stored), Some(link_hash)) = (stored, link_hash) else {
            redis.query::<i64>(&Cmd::ttl(&key)).await?;
            redis.query::<i64>(&Cmd::ttl(&failures)).await?;
            return Ok(None);
        };
        let (attempt, ttl): (u32, i64) = redis
            .query_pipe(
                redis::pipe()
                    .atomic()
                    .hincr(&key, "code_attempts", 1)
                    .ttl(&key),
            )
            .await?;
        if ttl == -1 {
            // O pedido expirou entre os dois comandos, e o HINCRBY criou uma
            // chave só com a contagem, sem TTL.
            redis.query::<()>(&Cmd::del(&key)).await?;
            return Ok(None);
        }
        if attempt > CODE_ATTEMPTS {
            // Código morto não soma: travar o código de alguém custa um pedido
            // (um email) a cada 5 erros.
            redis.query::<i64>(&Cmd::ttl(&failures)).await?;
            return Ok(None);
        }
        let (failed,): (u32,) = redis
            .query_pipe(
                redis::pipe()
                    .atomic()
                    .incr(&failures, 1)
                    .expire(&failures, CODE_FAILURES_TTL_SECS)
                    .ignore(),
            )
            .await?;
        if failed > CODE_FAILURES {
            return Ok(None);
        }
        let typed = code_hash(&link_hash, code);
        if !token::constant_time_eq(typed.as_bytes(), stored.as_bytes()) {
            return Ok(None);
        }
        // Sem o pedido aqui, o link foi aberto no meio tempo.
        Ok(self
            .take(redis, &link_hash)
            .await?
            .map(|(_, fields)| fields))
    }

    /// Derruba o pedido de `owner`, se houver: o link e o código param de valer.
    pub async fn cancel(self, redis: &Redis, owner: &str) -> RedisResult<()> {
        let key = self.record_key(redis, owner);
        let token: Option<String> = redis.query(&Cmd::hget(&key, "token")).await?;
        let mut pipe = redis::pipe();
        pipe.atomic().del(&key).ignore();
        if let Some(token) = token {
            pipe.del(self.pointer_key(redis, &token)).ignore();
        }
        redis.query_pipe(&pipe).await
    }

    /// O pedido sai do Redis. Usá-lo prova que o dono recebe no email, então os
    /// erros somados dele zeram.
    async fn take(
        self,
        redis: &Redis,
        hash: &str,
    ) -> RedisResult<Option<(String, HashMap<String, String>)>> {
        let owner: Option<String> = redis
            .query(&Cmd::get_del(self.pointer_key(redis, hash)))
            .await?;
        let Some(owner) = owner else {
            return Ok(None);
        };
        let key = self.record_key(redis, &owner);
        let (fields,): (HashMap<String, String>,) = redis
            .query_pipe(redis::pipe().atomic().hgetall(&key).del(&key).ignore())
            .await?;
        if fields.get("token").map(String::as_str) != Some(hash) {
            return Ok(None);
        }
        redis
            .query::<()>(&Cmd::del(failures_key(redis, &owner)))
            .await?;
        Ok(Some((owner, fields)))
    }
}

fn failures_key(redis: &Redis, owner: &str) -> String {
    redis.key(format_args!("email_code_failures:{owner}"))
}

/// Hash do código, com o hash do token do mesmo pedido como sal. Com um milhão
/// de códigos possíveis, o hash não esconde o código de quem lê o Redis; só o
/// tira do texto puro. O que segura são as tentativas.
fn code_hash(link_hash: &str, code: &str) -> String {
    token::hex(&token::hash_token(&format!("{link_hash}:{code}")))
}
