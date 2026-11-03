//! Ler, entrar e agir na sala de veto. O token do caminho é a credencial; só
//! entrar lê a sessão (para o nome da conta). As ações devolvem a sala como
//! ficou, sem reler o banco.

use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use engine::veto::Event;
use sqlx::PgConnection;

use super::{
    room::Room,
    types::{JoinVetoRoom, VetoRoomDetail, VetoRoomPath, VetoRoomRole},
};
use crate::{
    AppState,
    auth::CurrentUser,
    matches::{
        Side,
        veto::{Ctx, Kind, NewVetoEvent, VetoChoice, ev_side, step},
    },
    outbox,
    shared::{ApiError, ErrorBody, JsonBody, PathParams, text::clean_text},
};

/// Validade a partir do início.
const STARTED_HOURS: i32 = 24;
/// Tamanho do nome de um lugar, o mesmo do nome de uma conta e da tabela.
const MAX_NAME: usize = 32;

/// O espectador recebe o que a sala guardou em memória, com a hora de agora.
#[derive(serde::Serialize)]
struct Stamped<'a> {
    now: chrono::DateTime<Utc>,
    #[serde(flatten)]
    rest: &'a serde_json::Map<String, serde_json::Value>,
}

/// A sala do link `token`, depois dos sorteios vencidos (só abre transação, e
/// trava a sala, quando há algum).
async fn current(state: &AppState, token: &str) -> Result<Room, ApiError> {
    let now = Utc::now();
    let mut conn = state.db.acquire().await?;
    let room = Room::read(&mut conn, token).await?;
    if !room.is_due(now) {
        return Ok(room);
    }
    drop(conn);
    let mut tx = state.db.begin().await?;
    let mut room = Room::lock(&mut tx, token).await?;
    room.apply_due(&mut tx, now).await?;
    tx.commit().await?;
    if room.changed {
        state.veto_rooms.changed(room.id, None);
    }
    Ok(room)
}

#[utoipa::path(
    get,
    path = "/veto-rooms/{token}",
    tag = "veto_rooms",
    params(("token" = String, Path, description = "Um link da sala: de gestão, de espectador ou de um lugar")),
    responses(
        (status = OK, body = VetoRoomDetail, description = "A sala como este link a vê: o veto com as opções de quem joga e, no de gestão, todos os links. O espectador lê da memória enquanto nada muda, por muitos que sejam. Não lê a sessão"),
        (status = NOT_FOUND, body = ErrorBody, description = "veto_room.not_found (não existe ou venceu)"),
    )
)]
pub async fn get_veto_room(
    State(state): State<AppState>,
    PathParams(path): PathParams<VetoRoomPath>,
) -> Result<Response, ApiError> {
    if let Some(view) = state.veto_rooms.view(&path.token) {
        let body = view
            .current(|| async {
                let room = current(&state, &path.token).await?;
                Ok(room.detail(Utc::now()))
            })
            .await?;
        let stamped = Stamped {
            now: Utc::now(),
            rest: &body,
        };
        return Ok(Json(stamped).into_response());
    }
    let stamp = state.veto_rooms.stamp();
    let room = current(&state, &path.token).await?;
    let detail = room.detail(Utc::now());
    if room.role == VetoRoomRole::Spectator {
        state
            .veto_rooms
            .keep_view(path.token, room.id, stamp, &detail);
    }
    Ok(Json(detail).into_response())
}

/// Trava a sala do link `token` e aplica os sorteios vencidos: toda ação começa
/// assim, para valer contra a vez de agora. 404 `veto_room.not_found`.
async fn opened(conn: &mut PgConnection, token: &str) -> Result<Room, ApiError> {
    let mut room = Room::lock(&mut *conn, token).await?;
    room.apply_due(conn, Utc::now()).await?;
    Ok(room)
}

/// Encerra a transação da ação e avisa quem está na sala, se algo mudou.
async fn finish(
    state: &AppState,
    tx: sqlx::Transaction<'_, sqlx::Postgres>,
    room: &Room,
) -> Result<Json<VetoRoomDetail>, ApiError> {
    tx.commit().await?;
    if room.changed {
        state.veto_rooms.changed(room.id, outbox::origin());
    }
    Ok(Json(room.detail(Utc::now())))
}

#[utoipa::path(
    post,
    path = "/veto-rooms/{token}/join",
    tag = "veto_rooms",
    params(("token" = String, Path, description = "O link de um lugar")),
    request_body = Option<JoinVetoRoom>,
    responses(
        (status = OK, body = VetoRoomDetail, description = "Quem abre o link entra no lugar: logado, com o nome da conta; sem login, com o nome que digitou. O nome é definido uma vez: quem reabre o link, ou o abre em outro aparelho, continua como aquele lugar, e o nome enviado é ignorado"),
        (status = FORBIDDEN, body = ErrorBody, description = "veto_room.not_player (link de gestão ou de espectador)"),
        (status = NOT_FOUND, body = ErrorBody, description = "veto_room.not_found (não existe ou venceu)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "veto_room.name_invalid (sem login e sem nome, ou nome fora de 1 a 32 caracteres numa linha), request.invalid_body"),
    )
)]
pub async fn join_veto_room(
    State(state): State<AppState>,
    user: Option<CurrentUser>,
    PathParams(path): PathParams<VetoRoomPath>,
    body: Option<JsonBody<JoinVetoRoom>>,
) -> Result<Json<VetoRoomDetail>, ApiError> {
    const INVALID: ApiError = ApiError::invalid("veto_room.name_invalid");
    let mut tx = state.db.begin().await?;
    let mut room = opened(&mut tx, &path.token).await?;
    let seat = room
        .seat()
        .ok_or(ApiError::forbidden("veto_room.not_player"))?;
    if seat.name.is_none() {
        let seat = seat.id;
        let name = match (&user, body) {
            (Some(user), _) => user.display_name.chars().take(MAX_NAME).collect(),
            (None, Some(JsonBody(JoinVetoRoom { name: Some(name) }))) => {
                clean_text(&name, MAX_NAME, INVALID.code)?
            }
            (None, _) => return Err(INVALID),
        };
        room.set_name(&mut tx, seat, name).await?;
    }
    finish(&state, tx, &room).await
}

#[utoipa::path(
    post,
    path = "/veto-rooms/{token}/ready",
    tag = "veto_rooms",
    params(("token" = String, Path, description = "O link de um lugar")),
    responses(
        (status = OK, body = VetoRoomDetail, description = "O lado de quem clica está pronto (qualquer jogador do lado responde por ele). Com os dois prontos, o veto começa: quem começa é sorteado, e a sala passa a valer 24 h. Repetir não muda nada"),
        (status = FORBIDDEN, body = ErrorBody, description = "veto_room.not_player (link de gestão ou de espectador)"),
        (status = NOT_FOUND, body = ErrorBody, description = "veto_room.not_found (não existe ou venceu)"),
        (status = CONFLICT, body = ErrorBody, description = "veto_room.not_joined (falta entrar no lugar), veto.already_begun"),
    )
)]
pub async fn veto_room_ready(
    State(state): State<AppState>,
    PathParams(path): PathParams<VetoRoomPath>,
) -> Result<Json<VetoRoomDetail>, ApiError> {
    let mut tx = state.db.begin().await?;
    let mut room = opened(&mut tx, &path.token).await?;
    let (seat, side) = room.acting_seat()?;
    if room.log.state.begun() {
        return Err(ApiError::conflict("veto.already_begun"));
    }
    let [ready_a, ready_b] = room.log.ready();
    let (mine, theirs) = match side {
        Side::A => (ready_a, ready_b),
        Side::B => (ready_b, ready_a),
    };
    if !mine {
        let mut events = vec![NewVetoEvent::new(Kind::Ready, None, Some(side), Some(seat))];
        if theirs {
            events.push(begin(&room)?);
            room.extend(&mut tx, STARTED_HOURS).await?;
        }
        room.record(&mut tx, events).await?;
    }
    finish(&state, tx, &room).await
}

/// O começo: a seed da série e quem começa, por sorteio (a sala não tem seed
/// melhor). 422 `champions.pool_exhausted` `{required, available}` com campeões
/// ativos de menos para os bans.
fn begin(room: &Room) -> Result<NewVetoEvent, ApiError> {
    let seed: u64 = rand::random();
    let first = if rand::random::<bool>() {
        Side::A
    } else {
        Side::B
    };
    let ctx = Ctx::alone(&room.champions);
    engine::veto::Veto::new(room.rules.clone(), room.best_of()).apply(
        Event::Begin {
            seed,
            first: ev_side(first),
        },
        Some(&ctx.engine()),
    )?;
    Ok(NewVetoEvent {
        // O u64 gravado bit a bit num bigint.
        seed: Some(seed as i64),
        ..NewVetoEvent::new(Kind::Begin, Some(1), Some(first), None)
    })
}

#[utoipa::path(
    post,
    path = "/veto-rooms/{token}/steps",
    tag = "veto_rooms",
    params(("token" = String, Path, description = "O link de um lugar")),
    request_body = VetoChoice,
    responses(
        (status = OK, body = VetoRoomDetail, description = "O passo do lado da vez, como na série de um torneio: ban ou pick de mapa, ban ou pick de campeão; qualquer jogador do lado faz o passo dele. Na janela simultânea dos times (`team_pick`), cada lugar escolhe o próprio campeão de cada partida (`game`). Vale até 2 s depois do prazo que a sala mostra; depois, é sorteado. Não lê a sessão"),
        (status = FORBIDDEN, body = ErrorBody, description = "veto_room.not_player (link de gestão ou de espectador)"),
        (status = NOT_FOUND, body = ErrorBody, description = "veto_room.not_found (não existe ou venceu)"),
        (status = CONFLICT, body = ErrorBody, description = "veto_room.not_joined (falta entrar no lugar), veto.not_begun, veto.step_changed (veio `step` e a vez já não é essa: o tempo acabou ou o outro lado agiu), veto.finished, veto.not_your_turn, veto.already_picked, veto.champion_taken"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "veto.option_invalid, veto.champion_forbidden, request.invalid_body"),
    )
)]
pub async fn veto_room_step(
    State(state): State<AppState>,
    PathParams(path): PathParams<VetoRoomPath>,
    JsonBody(body): JsonBody<VetoChoice>,
) -> Result<Json<VetoRoomDetail>, ApiError> {
    let mut tx = state.db.begin().await?;
    let mut room = opened(&mut tx, &path.token).await?;
    let (seat, side) = room.acting_seat()?;
    let state_now = &room.log.state;
    if !state_now.begun() {
        return Err(ApiError::conflict("veto.not_begun"));
    }
    step::seen(&room.log, body.step)?;
    let choice = step::choice(&room.maps, &room.champions, &body)?;
    let turn = step::open_turn(state_now)?;
    let ctx = Ctx::alone(&room.champions);
    let event = step::planned(state_now, &ctx, turn, side, seat, choice, body.game)?;
    room.record(&mut tx, vec![event]).await?;
    finish(&state, tx, &room).await
}
