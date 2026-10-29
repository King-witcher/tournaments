//! Seeding antes de iniciar: o organizador vê os times elegíveis na ordem
//! que vale, sorteia (a seed fica gravada no torneio e reproduz a ordem) e
//! ajusta à mão. Ao iniciar, `finalize` fecha a ordem que entra na chave. O
//! seeding por rating é a ordem manual preenchida pela tela.

use axum::{Json, extract::State};
use engine::core::ParticipantId;
use sqlx::PgConnection;

use super::{
    db,
    seed_order::{self, Entrant},
    types::{SeedingOrder, Team},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, PathParams},
    tenants::{Organizer, TenantMember},
    tournaments::{self, StoredTournament, TournamentPath},
};

#[utoipa::path(
    get,
    path = "/tenants/{slug}/tournaments/{tournament}/seeding",
    tag = "teams",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = OK, body = Vec<Team>, description = "Times elegíveis (com o check-in de todos os jogadores; sem check-in no torneio, todos) na ordem que vale ao iniciar: os com seed pelo seed, depois os sem, na ordem de criação (no duelo, a de inscrição)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
    )
)]
pub async fn get_seeding(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<Json<Vec<Team>>, ApiError> {
    let mut conn = state.db.acquire().await?;
    let tournament = tournaments::find(&mut conn, member.tenant, &path.tournament, None)
        .await?
        .ok_or(ApiError::not_found("tournament.not_found"))?;
    let order = seed_order::current_order(eligible(&mut conn, &tournament).await?);
    Ok(Json(in_order(&mut conn, &tournament, &order).await?))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/seeding/shuffle",
    tag = "teams",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = OK, body = Vec<Team>, description = "Times elegíveis na ordem sorteada, já gravada como seeds 1..N; a seed do sorteio fica no torneio"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.participants_locked (já começou)"),
    )
)]
pub async fn shuffle_seeding(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<Json<Vec<Team>>, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = editable(&mut tx, &member, &path.tournament).await?;
    let eligible = eligible(&mut tx, &tournament).await?;
    let order = draw(&mut tx, &tournament, eligible).await?;
    let seeded = in_order(&mut tx, &tournament, &order).await?;
    tournaments::changed(&mut tx, tournament.id).await?;
    tx.commit().await?;
    Ok(Json(seeded))
}

#[utoipa::path(
    put,
    path = "/tenants/{slug}/tournaments/{tournament}/seeding",
    tag = "teams",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    request_body = SeedingOrder,
    responses(
        (status = OK, body = Vec<Team>, description = "Ordem manual gravada como seeds 1..N"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.participants_locked (já começou)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "seeding.order_invalid (não são exatamente os times elegíveis, cada um uma vez), request.invalid_body"),
    )
)]
pub async fn put_seeding(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
    JsonBody(body): JsonBody<SeedingOrder>,
) -> Result<Json<Vec<Team>>, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = editable(&mut tx, &member, &path.tournament).await?;
    let order = seed_order::manual_order(eligible(&mut tx, &tournament).await?, &body.team_ids)?;
    db::save_seeds(&mut tx, tournament.id, &seed_order::ids(&order)).await?;
    let seeded = in_order(&mut tx, &tournament, &order).await?;
    tournaments::changed(&mut tx, tournament.id).await?;
    tx.commit().await?;
    Ok(Json(seeded))
}

/// Fecha o seeding ao iniciar, no torneio que quem chama já travou: vale a
/// ordem gravada, os elegíveis sem seed vão para o fim (na ordem de criação)
/// e, se nenhum tem seed, sorteia (e grava a seed). Os seeds viram 1..N entre
/// os elegíveis, e os outros times ficam sem. Devolve a ordem que entra na
/// chave. 409 `tournament.not_enough_eligible` `{min, eligible}` com menos de
/// `min` elegíveis.
pub async fn finalize(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    min: usize,
) -> Result<Vec<ParticipantId>, ApiError> {
    let eligible = eligible(conn, tournament).await?;
    if eligible.len() < min {
        return Err(ApiError::conflict("tournament.not_enough_eligible")
            .with_params([("min", min as i64), ("eligible", eligible.len() as i64)]));
    }
    let order = if eligible.iter().all(|e| e.seed.is_none()) {
        draw(conn, tournament, eligible).await?
    } else {
        let order = seed_order::current_order(eligible);
        db::save_seeds(conn, tournament.id, &seed_order::ids(&order)).await?;
        order
    };
    Ok(order.iter().map(|e| ParticipantId(e.id)).collect())
}

/// O torneio travado, antes de começar (409 `tournament.participants_locked`
/// depois).
async fn editable(
    conn: &mut PgConnection,
    member: &TenantMember<Organizer>,
    slug: &str,
) -> Result<StoredTournament, ApiError> {
    let tournament = tournaments::find_for_update(conn, member.tenant, slug, None)
        .await?
        .ok_or(ApiError::not_found("tournament.not_found"))?;
    if !tournament.status.participants_editable() {
        return Err(ApiError::conflict("tournament.participants_locked"));
    }
    Ok(tournament)
}

async fn eligible(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
) -> Result<Vec<Entrant>, sqlx::Error> {
    let entrants = db::entrants(conn, tournament.id).await?;
    Ok(seed_order::eligible(
        entrants,
        tournament.schedule.has_check_in(),
    ))
}

/// Sorteia com uma seed nova e grava a seed e os seeds.
async fn draw(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    eligible: Vec<Entrant>,
) -> Result<Vec<Entrant>, sqlx::Error> {
    let seed: u64 = rand::random();
    let order = seed_order::drawn_order(eligible, seed);
    db::save_draw_seed(conn, tournament.id, seed).await?;
    db::save_seeds(conn, tournament.id, &seed_order::ids(&order)).await?;
    Ok(order)
}

/// Os times de `order`, nessa ordem, como a API os mostra.
async fn in_order(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    order: &[Entrant],
) -> Result<Vec<Team>, sqlx::Error> {
    let all = db::list(conn, tournament.id, false).await?;
    Ok(order
        .iter()
        .filter_map(|e| all.iter().find(|t| t.id == e.id).map(|t| t.view()))
        .collect())
}
