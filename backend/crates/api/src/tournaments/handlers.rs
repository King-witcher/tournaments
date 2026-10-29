use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;

use super::{
    changes, contest,
    db::{self, NewTournament},
    edit, prints,
    schedule::Schedule,
    setup::Setup,
    slug::validate_slug,
    status::TournamentStatus,
    texts::Texts,
    types::{
        CreateTournament, CreateTranslation, Tournament, TournamentPath, TournamentSummary,
        UpdateTournament,
    },
    veto,
    visibility::find_visible,
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, Locale, PathParams},
    tenants::{self, Organizer, Role, TenantMember},
};

const NOT_FOUND: ApiError = ApiError::not_found("tournament.not_found");

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments",
    tag = "tournaments",
    params(("slug" = String, Path, description = "Slug do tenant")),
    request_body = CreateTournament,
    responses(
        (status = CREATED, body = Tournament, description = "Torneio criado com as traduções enviadas: rascunho, ou já publicado (inscrição aberta, ou em check-in se a janela já abriu) com `publish: true`. Tudo numa transação só"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.slug_taken"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "tournament.slug_invalid, tournament.slug_reserved, tournament.name_invalid, tournament.description_invalid {max}, tournament.rules_invalid {max} (de uma tradução: com `params.locale`, o idioma dela), tournament.translation_same_locale {locale}, tournament.max_participants_invalid {min, max}, tournament.bans_invalid {max}, tournament.stage_count_invalid {max}, tournament.check_in_minutes_invalid {min, max}, tournament.contest_minutes_invalid {min, max}, tournament.veto_seconds_invalid {min, max}, tournament.map_unknown, tournament.map_pool_repeated, tournament.map_pool_too_small {min}, tournament.map_pool_too_large {max}, tournament.prints_unavailable (print obrigatório sem o bucket configurado), format.invalid_best_of {best_of}, champions.pool_exhausted {required, available}, request.invalid_body"),
    )
)]
pub async fn create_tournament(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    JsonBody(body): JsonBody<CreateTournament>,
) -> Result<(StatusCode, Json<Tournament>), ApiError> {
    validate_slug(&body.slug)?;
    let texts = Texts::clean(&body.name, &body.description, &body.rules)?;
    let mut conn = state.db.acquire().await?;
    let veto = veto::resolve(&mut conn, body.veto.unwrap_or_default(), None).await?;
    let setup = Setup::from_dto(
        body.max_participants,
        body.champion_rules,
        body.stages,
        veto,
    );
    setup.check(body.mode, db::champion_pool(&mut *conn).await?, 0)?;
    drop(conn);
    let schedule = Schedule {
        starts_at: body.starts_at,
        check_in_minutes: body.check_in_minutes,
    };
    schedule.check()?;
    contest::check(body.contest_minutes)?;
    let translations = clean_translations(body.locale, body.translations)?;
    prints::check_new(body.prints_required, state.storage.is_some())?;

    let new = NewTournament {
        slug: &body.slug,
        locale: body.locale,
        texts: &texts,
        mode: body.mode,
        visibility: body.visibility,
        setup: &setup,
        schedule,
        contest_minutes: body.contest_minutes,
        counts_for_rating: body.counts_for_rating,
        prints_required: body.prints_required,
    };
    let mut tx = state.db.begin().await?;
    let id = db::create(&mut tx, member.tenant, new).await?;
    for (locale, texts) in &translations {
        db::put_translation(&mut tx, id, *locale, texts).await?;
    }
    if body.publish {
        db::publish(&mut tx, id).await?;
    }
    changes::changed(&mut tx, id).await?;
    let created = db::find(&mut tx, member.tenant, &body.slug, Some(member.user.id))
        .await?
        .ok_or_else(ApiError::internal)?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(created.into())))
}

/// Limpa as traduções da criação com as regras do PUT. Erro de texto ou de
/// idioma sai com `params.locale` da tradução.
fn clean_translations(
    original: Locale,
    list: Vec<CreateTranslation>,
) -> Result<Vec<(Locale, Texts)>, ApiError> {
    let mut cleaned: Vec<(Locale, Texts)> = Vec::with_capacity(list.len());
    for t in list {
        let at = |e: ApiError| e.with_text_param("locale", t.locale.as_str());
        if t.locale == original {
            return Err(at(ApiError::invalid("tournament.translation_same_locale")));
        }
        if cleaned.iter().any(|(l, _)| *l == t.locale) {
            return Err(ApiError::invalid("request.invalid_body"));
        }
        let texts = Texts::clean(&t.name, &t.description, &t.rules).map_err(at)?;
        cleaned.push((t.locale, texts));
    }
    Ok(cleaned)
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/tournaments",
    tag = "tournaments",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = OK, body = Vec<TournamentSummary>, description = "Torneios do tenant, do mais novo ao mais antigo. Rascunhos e privados só para organizer e owner; privados também para os inscritos"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
    )
)]
pub async fn list_tournaments(
    State(state): State<AppState>,
    member: TenantMember,
) -> Result<Json<Vec<TournamentSummary>>, ApiError> {
    db::list(
        &state.db,
        member.tenant,
        member.user.id,
        member.role >= Role::Organizer,
    )
    .await
    .map(Json)
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/tournaments/{tournament}",
    tag = "tournaments",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = OK, body = Tournament, description = "O torneio, com fases e traduções"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found (inexistente; para player, também rascunho ou privado sem inscrição)"),
    )
)]
pub async fn get_tournament(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<Json<Tournament>, ApiError> {
    let found = find_visible(&mut *state.db.acquire().await?, &member, &path.tournament).await?;
    Ok(Json(found.into()))
}

#[utoipa::path(
    patch,
    path = "/tenants/{slug}/tournaments/{tournament}",
    tag = "tournaments",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    request_body = UpdateTournament,
    responses(
        (status = OK, body = Tournament, description = "Torneio atualizado; a checagem de campeões roda de novo se a configuração mudou"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.finished (textos, visibilidade, janela de contestação ou print obrigatório), tournament.setup_locked (vagas, campeões, fases, veto ou conta para o rating depois de começar), tournament.schedule_locked (data de início ou janela de check-in depois de começar), tournament.max_participants_below_registered {registered}"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "Os mesmos da criação, menos slug"),
    )
)]
pub async fn update_tournament(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
    JsonBody(mut body): JsonBody<UpdateTournament>,
) -> Result<Json<Tournament>, ApiError> {
    let viewer = Some(member.user.id);
    let mut tx = state.db.begin().await?;
    let current = db::find_for_update(&mut tx, member.tenant, &path.tournament, viewer)
        .await?
        .ok_or(NOT_FOUND)?;
    let veto = match body.veto.take() {
        Some(input) => Some(veto::resolve(&mut tx, input, Some(&current.setup.veto)).await?),
        None => None,
    };
    let prints_required = prints::edit(
        current.status,
        current.prints_required,
        body.prints_required,
        state.storage.is_some(),
    )?;
    let stored = edit::Current {
        status: current.status,
        texts: &current.texts,
        setup: &current.setup,
        schedule: current.schedule,
        contest_minutes: current.contest_minutes,
        counts_for_rating: current.counts_for_rating,
    };
    let edit = edit::apply(stored, veto, body)?;
    if let Some(setup) = &edit.setup {
        let pool = db::champion_pool(&mut *tx).await?;
        setup.check(current.mode, pool, current.participants)?;
    }
    db::update(&mut tx, current.id, &edit).await?;
    if let Some(required) = prints_required {
        db::set_prints_required(&mut tx, current.id, required).await?;
    }
    changes::changed(&mut tx, current.id).await?;
    if current.schedule.start_due(current.status, Utc::now()) {
        // Adiado, sai do que espera a organização (se continua, o evento é o
        // mesmo de `changed` e entra uma vez).
        tenants::organizing_changed(&mut tx, member.tenant).await?;
    }
    let updated = db::find(&mut tx, member.tenant, &path.tournament, viewer)
        .await?
        .ok_or_else(ApiError::internal)?;
    tx.commit().await?;
    Ok(Json(updated.into()))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/publish",
    tag = "tournaments",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = OK, body = Tournament, description = "Rascunho publicado: a inscrição abre e os membros passam a ver o torneio (o privado, só os inscritos). Se há data e a janela de check-in já abriu, vem em check-in; sem data, em inscrição"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_draft"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "champions.pool_exhausted {required, available} (o pool mudou desde a criação)"),
    )
)]
pub async fn publish_tournament(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<Json<Tournament>, ApiError> {
    let viewer = Some(member.user.id);
    let mut tx = state.db.begin().await?;
    let current = db::find_for_update(&mut tx, member.tenant, &path.tournament, viewer)
        .await?
        .ok_or(NOT_FOUND)?;
    if current.status != TournamentStatus::Draft {
        return Err(ApiError::conflict("tournament.not_draft"));
    }
    let pool = db::champion_pool(&mut *tx).await?;
    current
        .setup
        .check(current.mode, pool, current.participants)?;
    db::publish(&mut tx, current.id).await?;
    changes::changed(&mut tx, current.id).await?;
    let published = db::find(&mut tx, member.tenant, &path.tournament, viewer)
        .await?
        .ok_or_else(ApiError::internal)?;
    tx.commit().await?;
    Ok(Json(published.into()))
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/tournaments/{tournament}",
    tag = "tournaments",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = NO_CONTENT, description = "Rascunho excluído, com os inscritos e os links de inscrição dele"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_draft (só rascunho pode ser excluído)"),
    )
)]
pub async fn delete_tournament(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<StatusCode, ApiError> {
    let mut tx = state.db.begin().await?;
    let found = db::find(&mut tx, member.tenant, &path.tournament, None)
        .await?
        .ok_or(NOT_FOUND)?;
    // Antes de excluir: os inscritos e o endereço ainda existem.
    changes::changed(&mut tx, found.id).await?;
    if !db::delete_draft(&mut tx, found.id).await? {
        return Err(ApiError::conflict("tournament.not_draft"));
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
