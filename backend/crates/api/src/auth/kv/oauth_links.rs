//! Vínculos de provedor autorizados: `oauth_link:{hash da sessão}` guarda o
//! provedor e o challenge do OAuth que `/authorize` abriu para vincular nesta
//! sessão (depois de conferir a senha, se a conta tem), pelo prazo do fluxo.
//! O `/link` o consome (`GETDEL`), então só liga o code desse fluxo, nessa
//! sessão e uma vez. Um fluxo novo de vincular na mesma sessão troca o
//! anterior.

use redis::{Cmd, RedisResult};

use crate::shared::Redis;

fn link_key(redis: &Redis, session: &str) -> String {
    redis.key(format_args!("oauth_link:{session}"))
}

/// Autoriza a sessão de hash `session` (em hexadecimal) a vincular `provider`
/// com o code do fluxo de `challenge`, por `ttl_secs`.
pub async fn authorize_link(
    redis: &Redis,
    session: &str,
    provider: &str,
    challenge: &str,
    ttl_secs: u64,
) -> RedisResult<()> {
    redis
        .query(&Cmd::set_ex(
            link_key(redis, session),
            format!("{provider} {challenge}"),
            ttl_secs,
        ))
        .await
}

/// Gasta a autorização da sessão e diz se ela era para `provider` com
/// `challenge`.
pub async fn take_link_authorization(
    redis: &Redis,
    session: &str,
    provider: &str,
    challenge: &str,
) -> RedisResult<bool> {
    let grant: Option<String> = redis.query(&Cmd::get_del(link_key(redis, session))).await?;
    Ok(grant.is_some_and(|grant| grant == format!("{provider} {challenge}")))
}
