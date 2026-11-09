use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;

use super::{
    changes, contest,
    db::{self, NewTournament},
    rules_images,
    schedule::Schedule,
    screenshots,
    setup::Setup,
    slug::validate_slug,
    status::TournamentStatus,
    team_rules::TeamRules,
    texts::Texts,
    types::{
        CreateTournament, CreateTranslation, DeleteTournament, Tournament, TournamentPath,
        TournamentSummary,
    },
    veto,
    visibility::find_visible,
};
use crate::{
    AppState, notifications,
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
        (status = CREATED, body = Tournament, description = "Torneio criado com as traduções enviadas: rascunho, ou já publicado (inscrição aberta) com `publish: true`. Tudo numa transação só"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.slug_taken"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "tournament.slug_invalid, tournament.slug_reserved, tournament.name_invalid, tournament.description_invalid {max}, tournament.rules_invalid {max} (de uma tradução: com `params.locale`, o idioma dela), tournament.rules_image_invalid (imagem nas regras que não é uma enviada pela comunidade em `.../rules/images/uploads`, ou `<img` em HTML; de uma tradução, com `params.locale`), tournament.rules_images_too_many {max} (imagens distintas no torneio, somando o original e as traduções), tournament.translation_same_locale {locale}, tournament.team_size_invalid {min, max}, tournament.max_participants_invalid {min, max} (no TDM, o mínimo é o de dois times), tournament.bans_invalid {max}, tournament.stage_count_invalid {max}, tournament.contest_minutes_invalid {min, max}, tournament.veto_seconds_invalid {min, max}, tournament.team_pick_seconds_invalid {min, max}, tournament.map_unknown, tournament.map_pool_repeated, tournament.map_pool_too_small {min}, tournament.map_pool_too_large {max}, tournament.pick_order_invalid (a janela aberta não se escolhe mais), tournament.blind_needs_mirror (às cegas sem espelho), tournament.screenshots_unavailable (screenshot obrigatório pedido sem o bucket configurado), format.invalid_best_of {best_of, stage}, format.invalid_rounds {max, got, stage}, format.invalid_tiebreakers {stage} (desempate repetido, ou buchholz fora do suíço), format.too_many_participants {max, got, stage}, stage.elimination_not_last {stage} (eliminação só na última fase), stage.invalid_advance {stage, min, max} (corte de 2 ao teto do formato da fase), stage.entry_rules_first {stage} (corte ou check-in na fase 1), champions.pool_exhausted {required, available}, request.invalid_body"),
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
        body.mode,
        body.team_size,
        body.max_participants,
        body.champion_rules.into(),
        body.stages,
        veto,
        body.team_rules.unwrap_or(TeamRules::BALANCED),
    )?;
    setup.check(db::champion_pool(&mut *conn).await?, 0)?;
    drop(conn);
    let schedule = Schedule {
        starts_at: body.starts_at,
        check_in: body.check_in,
    };
    contest::check(body.contest_minutes)?;
    let translations = clean_translations(body.locale, body.translations)?;
    let screenshots_required =
        screenshots::new(body.screenshots_required, state.storage.is_some())?;

    let new = NewTournament {
        slug: &body.slug,
        locale: body.locale,
        texts: &texts,
        admission: body.admission,
        setup: &setup,
        schedule,
        contest_minutes: body.contest_minutes,
        counts_for_rating: body.counts_for_rating,
        screenshots_required,
    };
    let cdn = rules_images::cdn(&state);
    let mut tx = state.db.begin().await?;
    let id = db::create(&mut tx, member.tenant, new).await?;
    rules_images::save(&mut tx, member.tenant, id, None, &texts.rules, cdn).await?;
    for (locale, texts) in &translations {
        db::put_translation(&mut tx, id, *locale, texts).await?;
        rules_images::save(&mut tx, member.tenant, id, Some(*locale), &texts.rules, cdn)
            .await
            .map_err(|e| e.with_text_param("locale", locale.as_str()))?;
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
        (status = OK, body = Vec<TournamentSummary>, description = "Torneios do tenant, do mais novo ao mais antigo. Rascunhos só para organizer e owner"),
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
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found (inexistente; para player, também rascunho)"),
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
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/publish",
    tag = "tournaments",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = OK, body = Tournament, description = "Rascunho publicado: a inscrição abre e os membros passam a ver o torneio"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_draft"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "champions.pool_exhausted {required, available} (o pool mudou desde a criação), tournament.blind_needs_mirror"),
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
    current.setup.check(pool, current.participants)?;
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
    path = "/tenants/{slug}/tournaments/{tournament}/publish",
    tag = "tournaments",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = OK, body = Tournament, description = "De volta ao rascunho: a inscrição fecha, os membros deixam de ver o torneio e o endereço volta a mudar. Só com a inscrição aberta (fora do check-in) e nenhum inscrito"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_registration, tournament.has_participants"),
    )
)]
pub async fn unpublish_tournament(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<Json<Tournament>, ApiError> {
    let viewer = Some(member.user.id);
    let mut tx = state.db.begin().await?;
    let current = db::find_for_update(&mut tx, member.tenant, &path.tournament, viewer)
        .await?
        .ok_or(NOT_FOUND)?;
    if current.status != TournamentStatus::Registration {
        return Err(ApiError::conflict("tournament.not_registration"));
    }
    if current.participants > 0 {
        return Err(ApiError::conflict("tournament.has_participants"));
    }
    db::set_status(&mut tx, current.id, TournamentStatus::Draft).await?;
    changes::changed(&mut tx, current.id).await?;
    if current.schedule.start_due(current.status, Utc::now()) {
        // Saiu do que espera a organização, e `changed` já não o vê lá.
        tenants::organizing_changed(&mut tx, member.tenant).await?;
    }
    let draft = db::find(&mut tx, member.tenant, &path.tournament, viewer)
        .await?
        .ok_or_else(ApiError::internal)?;
    tx.commit().await?;
    Ok(Json(draft.into()))
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/tournaments/{tournament}",
    tag = "tournaments",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    request_body = Option<DeleteTournament>,
    responses(
        (status = NO_CONTENT, description = "Torneio excluído na hora, com tudo o que tem dentro (as imagens das regras saem do bucket se nenhum outro torneio as usa); o endereço fica livre. Ninguém é avisado: as páginas abertas só recarregam, as notificações que levavam a ele somem e o ranking é recalculado. Antes de começar, organizer e owner; depois, inclusive encerrado, só o owner. Fora do rascunho, com `confirm_slug`"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tenant.deletion_scheduled"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "tournament.confirmation_mismatch (o slug digitado não confere), request.invalid_body"),
    )
)]
pub async fn delete_tournament(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
    body: Option<JsonBody<DeleteTournament>>,
) -> Result<StatusCode, ApiError> {
    let mut tx = state.db.begin().await?;
    let found = db::find_for_update(&mut tx, member.tenant, &path.tournament, None)
        .await?
        .ok_or(NOT_FOUND)?;
    if found.status.has_started() && member.role < Role::Owner {
        return Err(ApiError::forbidden("tenant.forbidden"));
    }
    if found.status != TournamentStatus::Draft {
        let typed = body.map(|JsonBody(b)| b.confirm_slug);
        if typed.as_deref().map(str::trim) != Some(found.slug.as_str()) {
            return Err(ApiError::invalid("tournament.confirmation_mismatch"));
        }
    }
    // Antes de excluir: os inscritos e o endereço ainda existem. A home da
    // organização também recarrega (mapa contestado, hora de iniciar).
    let unread =
        notifications::unread_under(&mut tx, member.tenant, &found.address().path()).await?;
    changes::changed(&mut tx, found.id).await?;
    tenants::organizing_changed(&mut tx, member.tenant).await?;
    db::delete(&mut tx, found.id).await?;
    notifications::recount(&mut tx, &unread).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
