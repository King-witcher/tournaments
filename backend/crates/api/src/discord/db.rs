//! SQL do Discord: a configuração de cada comunidade (`discord_webhooks`) e a
//! fila de mensagens (`discord_messages`).

use std::time::Duration;

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgConnection, PgPool};

use super::url::WebhookUrl;
use crate::{
    shared::{ApiError, Locale},
    tenants::TenantId,
};

/// A configuração de uma comunidade, com o token (que não sai da API).
pub struct Config {
    pub webhook_id: String,
    pub token: String,
    pub events: Vec<String>,
    pub mention_players: bool,
    pub locale: Locale,
    pub broken: bool,
}

/// O que se escolhe na configuração, fora o endereço.
#[derive(Clone, Copy)]
pub struct Choices<'a> {
    pub events: &'a [String],
    pub mention_players: bool,
    pub locale: Locale,
}

pub async fn get(db: &PgPool, tenant: TenantId) -> Result<Option<Config>, ApiError> {
    let row = sqlx::query!(
        r#"select webhook_id, token, events, mention_players, locale,
                  broken_at is not null as "broken!"
           from discord_webhooks where tenant_id = $1"#,
        tenant.get()
    )
    .fetch_optional(db)
    .await?;
    row.map(|r| {
        Ok(Config {
            webhook_id: r.webhook_id,
            token: r.token,
            events: r.events,
            mention_players: r.mention_players,
            locale: Locale::from_db(&r.locale)?,
            broken: r.broken,
        })
    })
    .transpose()
}

/// Grava o webhook novo com o resto da configuração; a marca de quebrado e a
/// espera de ritmo do anterior saem.
pub async fn put_with_url(
    db: &PgPool,
    tenant: TenantId,
    url: &WebhookUrl,
    choices: Choices<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"insert into discord_webhooks (tenant_id, webhook_id, token, events, mention_players, locale)
           values ($1, $2, $3, $4, $5, $6)
           on conflict (tenant_id) do update
           set webhook_id = excluded.webhook_id, token = excluded.token,
               events = excluded.events, mention_players = excluded.mention_players,
               locale = excluded.locale, broken_at = null, retry_at = null,
               updated_at = now()"#,
        tenant.get(),
        url.id,
        url.token,
        choices.events,
        choices.mention_players,
        choices.locale.as_str()
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Muda só a configuração do webhook que já existe. `false`: não há webhook.
pub async fn update(
    db: &PgPool,
    tenant: TenantId,
    choices: Choices<'_>,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        r#"update discord_webhooks
           set events = $2, mention_players = $3, locale = $4, updated_at = now()
           where tenant_id = $1"#,
        tenant.get(),
        choices.events,
        choices.mention_players,
        choices.locale.as_str()
    )
    .execute(db)
    .await?;
    Ok(done.rows_affected() == 1)
}

/// Tira o webhook e a fila dele.
pub async fn delete(db: &PgPool, tenant: TenantId) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "delete from discord_webhooks where tenant_id = $1",
        tenant.get()
    )
    .execute(db)
    .await?;
    Ok(())
}

/// O Discord disse que o webhook não existe mais: marca, e a fila dele não
/// tem para onde ir.
pub async fn mark_broken(
    db: &PgPool,
    tenant: TenantId,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;
    sqlx::query!(
        "update discord_webhooks set broken_at = $2 where tenant_id = $1 and broken_at is null",
        tenant.get(),
        now
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "delete from discord_messages where tenant_id = $1",
        tenant.get()
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

/// Um envio que deu certo prova que o webhook existe.
pub async fn clear_broken(db: &PgPool, tenant: TenantId) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update discord_webhooks set broken_at = null where tenant_id = $1",
        tenant.get()
    )
    .execute(db)
    .await?;
    Ok(())
}

/// O webhook só aceita mensagens a partir de `until` (limite de ritmo).
pub async fn hold(db: &PgPool, tenant: TenantId, until: DateTime<Utc>) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update discord_webhooks set retry_at = $2 where tenant_id = $1",
        tenant.get(),
        until
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Um aviso para a fila: o código, os parâmetros e o caminho do `Notice` ou
/// `Announce` do outbox.
pub struct Queued<'a> {
    pub code: &'a str,
    pub params: &'a Value,
    pub path: &'a str,
}

/// Põe o aviso na fila do webhook da comunidade `tenant` (o slug) se ela tem
/// um que funciona e com o aviso ligado; avisos iguais (mesma `merge_key`) se
/// juntam. Devolve se entrou.
pub async fn enqueue(
    conn: &mut PgConnection,
    tenant: &str,
    message: &Queued<'_>,
    users: &[i64],
    merge_key: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        r#"insert into discord_messages (tenant_id, code, params, path, users, merge_key)
           select w.tenant_id, $2, $3, $4, $5, $6
           from discord_webhooks w join tenants t on t.id = w.tenant_id
           where t.slug = $1 and w.broken_at is null and $2 = any(w.events)
           on conflict (tenant_id, merge_key) where merge_key is not null do update
           set users = (select coalesce(array_agg(distinct u order by u), '{}')
                        from unnest(discord_messages.users || excluded.users) as u)"#,
        tenant,
        message.code,
        message.params,
        message.path,
        users,
        merge_key
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() > 0)
}

/// Uma mensagem reservada para o envio, com o que o envio precisa do webhook.
pub struct Job {
    pub id: i64,
    pub tenant: TenantId,
    pub code: String,
    pub params: Value,
    pub path: String,
    pub users: Vec<i64>,
    /// Com esta, quantas vezes já foi reservada.
    pub attempts: i16,
    pub webhook_id: String,
    pub token: String,
    pub locale: Locale,
    pub mention_players: bool,
}

/// A mensagem mais antiga que pode sair em `now`: o webhook funciona e não
/// está em espera. Reserva por `lease` (ninguém mais a pega nesse tempo) e
/// conta a tentativa. Com o webhook de uma comunidade em espera, as
/// mensagens das outras passam.
pub async fn claim(
    db: &PgPool,
    now: DateTime<Utc>,
    lease: Duration,
) -> Result<Option<Job>, ApiError> {
    let row = sqlx::query!(
        r#"with next as (
               select m.id from discord_messages m
               join discord_webhooks w on w.tenant_id = m.tenant_id
               where m.not_before <= $1 and w.broken_at is null
                 and (w.retry_at is null or w.retry_at <= $1)
               order by m.id
               limit 1
               for update of m skip locked
           )
           update discord_messages m
           set not_before = $1::timestamptz + make_interval(secs => $2), attempts = m.attempts + 1
           from next, discord_webhooks w
           where m.id = next.id and w.tenant_id = m.tenant_id
           returning m.id as "id!", m.tenant_id as "tenant_id!", m.code as "code!",
                     m.params as "params!", m.path as "path!", m.users as "users!",
                     m.attempts as "attempts!", w.webhook_id as "webhook_id!",
                     w.token as "token!", w.locale as "locale!",
                     w.mention_players as "mention_players!""#,
        now,
        lease.as_secs_f64()
    )
    .fetch_optional(db)
    .await?;
    row.map(|r| {
        Ok(Job {
            id: r.id,
            tenant: TenantId::new(r.tenant_id),
            code: r.code,
            params: r.params,
            path: r.path,
            users: r.users,
            attempts: r.attempts,
            webhook_id: r.webhook_id,
            token: r.token,
            locale: Locale::from_db(&r.locale)?,
            mention_players: r.mention_players,
        })
    })
    .transpose()
}

/// Saiu (ou não vale mais): tira da fila.
pub async fn done(db: &PgPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!("delete from discord_messages where id = $1", id)
        .execute(db)
        .await?;
    Ok(())
}

/// Tenta de novo em `at`. `counts`: se a tentativa que acabou conta (o 429
/// não é culpa da mensagem).
pub async fn retry(
    db: &PgPool,
    id: i64,
    at: DateTime<Utc>,
    counts: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update discord_messages
         set not_before = $2, attempts = attempts - (case when $3 then 0 else 1 end)
         where id = $1",
        id,
        at,
        counts
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Tira da fila o que esperou mais que `max_age` (avisos velhos já não
/// servem: a partida pronta de ontem).
pub async fn drop_stale(
    db: &PgPool,
    now: DateTime<Utc>,
    max_age: Duration,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query!(
        "delete from discord_messages where created_at < $1::timestamptz - make_interval(secs => $2)",
        now,
        max_age.as_secs_f64()
    )
    .execute(db)
    .await?;
    Ok(done.rows_affected())
}

/// Quem a mensagem procura: apelido na comunidade e id do Discord (se a
/// conta vinculou), na ordem de `users`.
pub struct Person {
    pub nickname: String,
    pub discord_id: Option<String>,
}

pub async fn people(
    db: &PgPool,
    tenant: TenantId,
    users: &[i64],
) -> Result<Vec<Person>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"select tm.nickname,
                  (select i.provider_user_id from user_identities i
                   where i.user_id = tm.user_id and i.provider = 'discord') as discord_id
           from unnest($2::bigint[]) with ordinality as u(id, n)
           join tenant_members tm on tm.user_id = u.id and tm.tenant_id = $1
           order by u.n"#,
        tenant.get(),
        users
    )
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| Person {
            nickname: r.nickname,
            discord_id: r.discord_id,
        })
        .collect())
}

/// O nome do torneio `slug` da comunidade `tenant` no idioma `locale`, se ele
/// tem tradução.
pub async fn translated_name(
    db: &PgPool,
    tenant: TenantId,
    slug: &str,
    locale: Locale,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select tr.name from tournaments t
           join tournament_translations tr on tr.tournament_id = t.id and tr.locale = $3
           where t.tenant_id = $1 and t.slug = $2"#,
        tenant.get(),
        slug,
        locale.as_str()
    )
    .fetch_optional(db)
    .await
}
