use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;

use super::{
    catalog::{CATALOG, find},
    db::{self, Choices, Config},
    render,
    sender::Outcome,
    types::{AvailableEvent, DiscordSettings, PutDiscord, WebhookSettings},
    url::WebhookUrl,
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody},
    tenants::{Owner, TenantMember},
};

#[utoipa::path(
    get,
    path = "/tenants/{slug}/discord",
    tag = "discord",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = OK, body = DiscordSettings, description = "Os avisos que existem e o webhook da comunidade (`null` sem ele). O endereço e o token nunca saem"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só o owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
    )
)]
pub async fn get_discord(
    State(state): State<AppState>,
    owner: TenantMember<Owner>,
) -> Result<Json<DiscordSettings>, ApiError> {
    let config = db::get(&state.db, owner.tenant).await?;
    Ok(Json(settings(config)))
}

#[utoipa::path(
    put,
    path = "/tenants/{slug}/discord",
    tag = "discord",
    params(("slug" = String, Path, description = "Slug do tenant")),
    request_body = PutDiscord,
    responses(
        (status = OK, body = DiscordSettings, description = "Configuração gravada. Com `url`, o webhook é trocado (e deixa de estar quebrado); sem ela, só os avisos, o idioma e as menções mudam"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só o owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "discord.url_invalid (só https://discord.com/api/webhooks/{id}/{token}), discord.url_required (não há webhook e a `url` não veio), discord.event_unknown, request.invalid_body"),
    )
)]
pub async fn put_discord(
    State(state): State<AppState>,
    owner: TenantMember<Owner>,
    JsonBody(body): JsonBody<PutDiscord>,
) -> Result<Json<DiscordSettings>, ApiError> {
    let events = known_events(&body.events)?;
    let choices = Choices {
        events: &events,
        mention_players: body.mention_players,
        locale: body.locale,
    };
    match body.url.as_deref() {
        Some(text) => {
            let url = WebhookUrl::parse(text).ok_or(ApiError::invalid("discord.url_invalid"))?;
            db::put_with_url(&state.db, owner.tenant, &url, choices).await?;
        }
        None if !db::update(&state.db, owner.tenant, choices).await? => {
            return Err(ApiError::invalid("discord.url_required"));
        }
        None => {}
    }
    let config = db::get(&state.db, owner.tenant).await?;
    Ok(Json(settings(config)))
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/discord",
    tag = "discord",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = NO_CONTENT, description = "Webhook removido, com as mensagens que esperavam (também se não havia)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só o owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
    )
)]
pub async fn delete_discord(
    State(state): State<AppState>,
    owner: TenantMember<Owner>,
) -> Result<StatusCode, ApiError> {
    db::delete(&state.db, owner.tenant).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/discord/test",
    tag = "discord",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = NO_CONTENT, description = "A mensagem de teste chegou ao canal"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só o owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, discord.not_configured"),
        (status = CONFLICT, body = ErrorBody, description = "discord.webhook_gone (o Discord não conhece o webhook: a configuração fica marcada como quebrada)"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "discord.rate_limited {retry_after} (segundos)"),
        (status = BAD_GATEWAY, body = ErrorBody, description = "discord.unreachable (rede ou erro do Discord), discord.rejected (o Discord recusou a mensagem)"),
    )
)]
pub async fn test_discord(
    State(state): State<AppState>,
    owner: TenantMember<Owner>,
) -> Result<StatusCode, ApiError> {
    let config = db::get(&state.db, owner.tenant)
        .await?
        .ok_or(ApiError::not_found("discord.not_configured"))?;
    let message = render::test_message(
        &state.auth.web_origin,
        &owner.slug,
        &owner.name,
        config.locale,
    );
    let now = Utc::now();
    match state
        .webhooks
        .post(&config.webhook_id, &config.token, &message)
        .await
    {
        Outcome::Sent { .. } => {
            db::clear_broken(&state.db, owner.tenant).await?;
            Ok(StatusCode::NO_CONTENT)
        }
        Outcome::Gone => {
            db::mark_broken(&state.db, owner.tenant, now).await?;
            Err(ApiError::conflict("discord.webhook_gone"))
        }
        Outcome::RateLimited { retry_after } => {
            let seconds = i64::try_from(retry_after.as_secs() + 1).unwrap_or(i64::MAX);
            Err(
                ApiError::new(StatusCode::TOO_MANY_REQUESTS, "discord.rate_limited")
                    .with_params([("retry_after", seconds)]),
            )
        }
        Outcome::Rejected(status) => {
            tracing::warn!(%status, "o Discord recusou a mensagem de teste");
            Err(ApiError::new(StatusCode::BAD_GATEWAY, "discord.rejected"))
        }
        Outcome::Failed(error) => {
            tracing::warn!(%error, "o teste do webhook do Discord falhou");
            Err(ApiError::new(
                StatusCode::BAD_GATEWAY,
                "discord.unreachable",
            ))
        }
    }
}

/// Os avisos pedidos, sem repetir e na ordem do catálogo. Código que não
/// existe: 422.
fn known_events(codes: &[String]) -> Result<Vec<String>, ApiError> {
    if codes.iter().any(|c| find(c).is_none()) {
        return Err(ApiError::invalid("discord.event_unknown"));
    }
    Ok(CATALOG
        .iter()
        .filter(|s| codes.iter().any(|c| c == s.code))
        .map(|s| s.code.to_owned())
        .collect())
}

fn settings(config: Option<Config>) -> DiscordSettings {
    DiscordSettings {
        available: CATALOG
            .iter()
            .map(|s| AvailableEvent {
                code: s.code.to_owned(),
                default_on: s.default_on,
                addressed: s.to != super::catalog::To::Nobody,
            })
            .collect(),
        webhook: config.map(|c| WebhookSettings {
            webhook_id: c.webhook_id,
            broken: c.broken,
            events: c.events,
            locale: c.locale,
            mention_players: c.mention_players,
        }),
    }
}
