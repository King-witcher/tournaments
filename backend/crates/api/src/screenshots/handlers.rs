use std::time::Duration;

use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;
use sqlx::PgConnection;

use super::{
    changes,
    db::{self, GameRef},
    object::{self, ImageType},
    policy::{self, Actor, Slot},
    types::{
        GamePath, MatchPath, MatchScreenshots, NewUpload, Screenshot, ScreenshotAuthor,
        ScreenshotPath, ScreenshotSettings, ScreenshotUpload,
    },
    upload::{self, NOT_FOUND},
    view::{self, Limits},
};
use crate::{
    AppState,
    matches::{self, MatchGames},
    shared::{ApiError, ErrorBody, JsonBody, PathParams, Storage},
    tenants::{Role, TenantMember},
};

/// Tamanho máximo de um screenshot (o mesmo check está na tabela).
pub const MAX_BYTES: u32 = 2 * 1024 * 1024;
/// Screenshots anexados por mapa (o mesmo limite está no banco).
pub const MAX_PER_GAME: u8 = 3;
/// Validade da URL de upload.
const UPLOAD_EXPIRES: Duration = Duration::from_secs(10 * 60);
/// Uploads pedidos e não registrados por pessoa numa comunidade, por hora:
/// o que passa disso é abuso do bucket, não screenshot de partida.
const MAX_PENDING_PER_HOUR: i64 = 10;

const LIMITS: Limits = Limits {
    max_bytes: MAX_BYTES,
    max_per_game: MAX_PER_GAME,
};
const NOT_CONFIGURED: ApiError = ApiError::new(
    StatusCode::SERVICE_UNAVAILABLE,
    "screenshots.not_configured",
);

fn storage(state: &AppState) -> Result<&Storage, ApiError> {
    state.storage.as_ref().ok_or(NOT_CONFIGURED)
}

fn actor<R>(member: &TenantMember<R>) -> Actor {
    Actor {
        user_id: member.user.id,
        organizer: member.role >= Role::Organizer,
    }
}

fn game_ref(m: &MatchGames, number: i16) -> GameRef<'_> {
    GameRef {
        tenant: m.tournament.tenant,
        tournament: m.tournament.id,
        stage: m.stage,
        key: &m.key,
        number,
    }
}

/// Os erros de `policy::attach` e 409 `screenshots.limit_reached` `{max}` com o
/// mapa cheio.
async fn check_attach(
    conn: &mut PgConnection,
    m: &MatchGames,
    actor: Actor,
    number: i16,
) -> Result<(), ApiError> {
    let slot = Slot::of(m, number);
    policy::attach(m.tournament.status, m.plays(actor.user_id), actor, slot)?;
    if db::attached_count(conn, &game_ref(m, number)).await? >= i64::from(MAX_PER_GAME) {
        return Err(ApiError::conflict("screenshots.limit_reached")
            .with_params([("max", i64::from(MAX_PER_GAME))]));
    }
    Ok(())
}

#[utoipa::path(
    get,
    path = "/screenshots/settings",
    tag = "screenshots",
    responses(
        (status = OK, body = ScreenshotSettings, description = "Se há screenshots nesta API (o bucket está configurado) e os limites de cada arquivo. Pública, sem sessão"),
    )
)]
pub async fn get_screenshot_settings(State(state): State<AppState>) -> Json<ScreenshotSettings> {
    Json(ScreenshotSettings {
        enabled: state.storage.is_some(),
        max_bytes: MAX_BYTES,
        max_per_game: MAX_PER_GAME,
        content_types: ImageType::ALL.to_vec(),
    })
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/screenshots",
    tag = "screenshots",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1"),
    ),
    responses(
        (status = OK, body = MatchScreenshots, description = "Os screenshots de cada mapa enviado e do próximo, com o que quem vê pode fazer. Sem o bucket configurado, `enabled` falso e nenhum mapa"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
    )
)]
pub async fn list(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<MatchPath>,
) -> Result<Json<MatchScreenshots>, ApiError> {
    let mut conn = state.db.acquire().await?;
    let m = matches::match_games(&mut conn, &member, &path.tournament, &path.key).await?;
    let Some(storage) = &state.storage else {
        return Ok(Json(view::disabled(LIMITS)));
    };
    let t = &m.tournament;
    let rows = db::of_match(&mut conn, t.tenant, t.id, m.stage, &m.key).await?;
    let screenshots =
        view::match_screenshots(&m, &rows, storage, actor(&member), LIMITS, Utc::now());
    Ok(Json(screenshots))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/screenshots",
    tag = "screenshots",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1"),
        ("game" = i16, Path, description = "Número do mapa na série: um enviado ou o próximo"),
    ),
    request_body = NewUpload,
    responses(
        (status = CREATED, body = ScreenshotUpload, description = "URL para o navegador mandar o arquivo direto ao bucket (PUT com o Content-Type dado e exatamente `size` bytes, em 10 minutos); depois, registrar com `.../screenshots/{id}/attach`. Anexam os dois jogadores (até o mapa confirmar) e organizer ou owner, com o torneio em andamento"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "screenshots.forbidden (não joga a partida nem organiza)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress, screenshots.game_unavailable (mapa que nem foi enviado nem é o próximo), screenshots.game_confirmed (jogador, mapa já confirmado), screenshots.limit_reached {max}"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "screenshots.size_invalid {max} (bytes), request.invalid_body (tipo fora de image/webp e image/jpeg)"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "screenshots.too_many_uploads {max} (uploads pedidos e não registrados na última hora)"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "screenshots.not_configured (a API está sem o bucket)"),
    )
)]
pub async fn create_upload(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<GamePath>,
    JsonBody(body): JsonBody<NewUpload>,
) -> Result<(StatusCode, Json<ScreenshotUpload>), ApiError> {
    let storage = storage(&state)?;
    if !(1..=MAX_BYTES).contains(&body.size) {
        return Err(ApiError::invalid("screenshots.size_invalid")
            .with_params([("max", i64::from(MAX_BYTES))]));
    }
    let mut tx = state.db.begin().await?;
    let m = matches::lock_match_games(&mut tx, &member, &path.tournament, &path.key).await?;
    let actor = actor(&member);
    check_attach(&mut tx, &m, actor, path.game).await?;
    let hour_ago = Utc::now() - chrono::Duration::hours(1);
    if db::pending_since(&mut tx, member.tenant, actor.user_id, hour_ago).await?
        >= MAX_PENDING_PER_HOUR
    {
        return Err(ApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "screenshots.too_many_uploads",
        )
        .with_params([("max", MAX_PENDING_PER_HOUR)]));
    }
    let kind = body.content_type;
    let place = object::Place {
        tenant: &member.slug,
        tournament: &m.tournament.slug,
        key: &m.key,
        number: path.game,
    };
    let key = object::new_key(&place, kind);
    let at = game_ref(&m, path.game);
    let id = db::insert_upload(&mut tx, &at, &key, kind, body.size, actor.user_id).await?;
    tx.commit().await?;
    let now = Utc::now();
    let url = storage.put_url(&key, kind.mime(), body.size.into(), UPLOAD_EXPIRES, now);
    let upload = ScreenshotUpload {
        id,
        url: url.into(),
        content_type: kind,
        expires_at: now + UPLOAD_EXPIRES,
    };
    Ok((StatusCode::CREATED, Json(upload)))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/screenshots/{screenshot}/attach",
    tag = "screenshots",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1"),
        ("game" = i16, Path, description = "Número do mapa na série"),
        ("screenshot" = i64, Path, description = "O `id` do upload"),
    ),
    responses(
        (status = OK, body = Screenshot, description = "Upload conferido no bucket (tipo e tamanho pedidos) e anexado ao mapa: aparece para todos e conta para o screenshot obrigatório"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "screenshots.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found, screenshots.not_found (upload de outra pessoa, de outro mapa, já registrado ou vencido)"),
        (status = CONFLICT, body = ErrorBody, description = "screenshots.upload_missing (o arquivo não chegou ao bucket), tournament.not_in_progress, screenshots.game_unavailable, screenshots.game_confirmed, screenshots.limit_reached {max}"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "screenshots.upload_invalid (chegou outro tipo ou tamanho: o upload é descartado)"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "screenshots.not_configured"),
    )
)]
pub async fn attach(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<ScreenshotPath>,
) -> Result<Json<Screenshot>, ApiError> {
    let storage = storage(&state)?;
    let pending = upload::verify(
        &state.db,
        storage,
        &member,
        &path.tournament,
        path.screenshot,
    )
    .await?;
    let mut tx = state.db.begin().await?;
    let m = matches::lock_match_games(&mut tx, &member, &path.tournament, &path.key).await?;
    if !pending.is_of(m.stage, &m.key, path.game) {
        return Err(NOT_FOUND);
    }
    let actor = actor(&member);
    check_attach(&mut tx, &m, actor, path.game).await?;
    let attached_at = db::attach(&mut tx, pending.id).await?.ok_or(NOT_FOUND)?;
    changes::changed(&mut tx, &member, &m).await?;
    tx.commit().await?;
    let slot = Slot::of(&m, path.game);
    Ok(Json(Screenshot {
        id: pending.id,
        url: storage.get_url(&pending.object_key, Utc::now()).into(),
        content_type: pending.content_type,
        uploaded_by: Some(ScreenshotAuthor {
            user_id: actor.user_id,
            nickname: member.nickname.clone(),
        }),
        attached_at,
        can_remove: policy::remove(actor, Some(actor.user_id), slot).is_ok(),
    }))
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/screenshots/{screenshot}",
    tag = "screenshots",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1"),
        ("game" = i16, Path, description = "Número do mapa na série"),
        ("screenshot" = i64, Path, description = "O `id` do screenshot"),
    ),
    responses(
        (status = NO_CONTENT, description = "Screenshot removido (o arquivo sai do bucket na rotina diária). Quem anexou remove o próprio enquanto o mapa não confirmou; organizer e owner, qualquer um"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "screenshots.forbidden (screenshot de outra pessoa)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found, screenshots.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "screenshots.game_confirmed (o próprio screenshot, com o mapa já confirmado)"),
    )
)]
pub async fn remove(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<ScreenshotPath>,
) -> Result<StatusCode, ApiError> {
    let mut tx = state.db.begin().await?;
    let m = matches::lock_match_games(&mut tx, &member, &path.tournament, &path.key).await?;
    let t = &m.tournament;
    let screenshot = db::find(&mut tx, t.tenant, t.id, path.screenshot)
        .await?
        .filter(|p| p.is_of(m.stage, &m.key, path.game))
        .ok_or(NOT_FOUND)?;
    policy::remove(
        actor(&member),
        screenshot.uploaded_by,
        Slot::of(&m, path.game),
    )?;
    db::delete(&mut tx, screenshot.id).await?;
    if screenshot.attached_at.is_some() {
        changes::changed(&mut tx, &member, &m).await?;
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
