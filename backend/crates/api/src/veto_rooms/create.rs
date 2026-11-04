//! Criar uma sala: os limites de quem cria e a configuração conferida. Os limites
//! são constantes aqui, e os do veto (prazos, pool) e os códigos deles são os do
//! torneio. O usuário não quer jogadores caindo no teto: ele só vale sem login, e
//! a interface oferece entrar com a conta.

use std::time::Duration;

use axum::{Json, extract::State, http::StatusCode};
use engine::champions::{ChampionBudget, NoRepeatRule, NoRepeatScope, check_budget};
use sqlx::PgConnection;

use super::{
    db::{self, NewRoom},
    types::{NewVetoRoom, VetoRoomActors, VetoRoomCreated, VetoRoomStart},
};
use crate::{
    AppState,
    auth::CurrentUser,
    matches::Side,
    shared::{ApiError, ClientIp, ErrorBody, JsonBody, token},
    tournaments::{MapMode, NoRepeatWho, VetoSetup, champion_pool, resolve_veto},
};

/// Salas criadas sem login valendo ao mesmo tempo. Conta logada nunca é barrada
/// por este teto.
const MAX_ANONYMOUS_ROOMS: i64 = 5_000;
/// Salas por hora, sem login, por IP (uma LAN sai pelo mesmo IP).
const ROOMS_PER_IP: u32 = 30;
/// Salas por hora, por conta.
const ROOMS_PER_ACCOUNT: u32 = 60;
const LIMIT_WINDOW: Duration = Duration::from_secs(3600);
/// Validade da sala em que ninguém começou.
const VALID_HOURS: i32 = 2;
/// Jogadores por lado, no máximo.
const MAX_TEAM_SIZE: i16 = 4;
/// Melhor de N aceitas.
const BEST_OF: [i16; 3] = [1, 3, 5];

/// O limite por hora: por conta, ou por IP sem login. 429
/// `veto_room.rate_limited` `{minutes}`.
fn limit(state: &AppState, user: Option<&CurrentUser>, ip: &ClientIp) -> Result<(), ApiError> {
    let (key, per_hour) = match user {
        Some(user) => (format!("veto_room:user:{}", user.id), ROOMS_PER_ACCOUNT),
        None => match ip.bucket() {
            Some(bucket) => (format!("veto_room:ip:{bucket}"), ROOMS_PER_IP),
            // Só no app dos testes: sem proxy e sem o endereço da conexão.
            None => return Ok(()),
        },
    };
    state
        .limits
        .hit(key.clone(), per_hour, LIMIT_WINDOW)
        .map_err(|wait| {
            tracing::warn!(key, ?wait, "limite de salas de veto por hora");
            let minutes = i64::try_from(wait.as_secs() / 60 + 1).unwrap_or(i64::MAX);
            ApiError::new(StatusCode::TOO_MANY_REQUESTS, "veto_room.rate_limited")
                .with_params([("minutes", minutes)])
        })
}

/// O teto de salas sem login. 503 `veto_room.anonymous_full`: quem entra com a
/// conta não é barrado.
async fn check_anonymous_ceiling(conn: &mut PgConnection) -> Result<(), ApiError> {
    if db::anonymous_active(conn).await? >= MAX_ANONYMOUS_ROOMS {
        tracing::warn!(
            max = MAX_ANONYMOUS_ROOMS,
            "teto de salas de veto sem login atingido"
        );
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "veto_room.anonymous_full",
        ));
    }
    Ok(())
}

#[utoipa::path(
    post,
    path = "/veto-rooms",
    tag = "veto_rooms",
    request_body = NewVetoRoom,
    responses(
        (status = CREATED, body = VetoRoomCreated, description = "A sala, com um link por jogador (centrada no capitão, um por time), um de espectador e um de gestão (`token`, que mostra todos). Qualquer um cria, sem login. Vale 2 h se ninguém começar e 24 h a partir do início"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "tournament.team_size_invalid {min, max}, format.invalid_best_of {best_of}, veto_room.map_mode_invalid (votação e sorteio ficam de fora), veto_room.no_phases (sem veto de mapas nem draft), tournament.veto_seconds_invalid {min, max}, tournament.team_pick_seconds_invalid {min, max}, tournament.map_unknown, tournament.map_pool_repeated, tournament.map_pool_too_small {min}, tournament.map_pool_too_large {max}, tournament.bans_invalid {max}, tournament.pick_order_invalid (a janela aberta não se escolhe mais), tournament.blind_needs_mirror (às cegas sem espelho), champions.pool_exhausted {required, available}, request.invalid_body"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "veto_room.rate_limited {minutes} (salas demais nesta hora, deste IP ou desta conta)"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "veto_room.anonymous_full (teto de salas sem login: com a conta, cria)"),
    )
)]
pub async fn create_veto_room(
    State(state): State<AppState>,
    user: Option<CurrentUser>,
    ip: ClientIp,
    JsonBody(body): JsonBody<NewVetoRoom>,
) -> Result<(StatusCode, Json<VetoRoomCreated>), ApiError> {
    limit(&state, user.as_ref(), &ip)?;
    let mut tx = state.db.begin().await?;
    let settings = settings(&mut tx, body).await?;
    if user.is_none() {
        check_anonymous_ceiling(&mut tx).await?;
    }
    let manage_token = token::random_token();
    let spectator_token = token::random_token();
    let seats = [Side::A, Side::B]
        .into_iter()
        .flat_map(|side| (1..=settings.team_size).map(move |position| (side, position)))
        .map(|(side, position)| {
            // Centrada no capitão, só o lugar 1 de cada lado tem link.
            let link = settings.who_acts == VetoRoomActors::All || position == 1;
            let token = if link {
                token::random_token()
            } else {
                String::new()
            };
            (side, position, token)
        })
        .collect();
    db::create(
        &mut tx,
        &NewRoom {
            created_by: user.as_ref().map(|u| u.id),
            manage_token: &manage_token,
            spectator_token: &spectator_token,
            team_size: settings.team_size,
            best_of: settings.best_of,
            map_mode: settings.veto.map_mode.as_str(),
            veto_order: settings.veto.order.as_str(),
            pick_order: settings.veto.pick_order.as_str(),
            who_starts: settings.who_starts.as_str(),
            team_actors: settings.who_acts.as_str(),
            champion_phase: settings.veto.champion_phase,
            bans_per_series: i16::from(settings.bans_per_series),
            mirror: settings.mirror,
            same_team: settings.same_team,
            no_repeat: settings.no_repeat.map(|who| match who {
                NoRepeatWho::Player => "player",
                NoRepeatWho::Team => "team",
            }),
            step_seconds: settings.veto.step_seconds,
            team_pick_seconds: settings.veto.team_pick_seconds,
            valid_hours: VALID_HOURS,
            pool: &settings.veto.pool,
            seats,
        },
    )
    .await?;
    tx.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(VetoRoomCreated {
            token: manage_token,
        }),
    ))
}

/// A configuração da sala, conferida.
struct Settings {
    team_size: i16,
    best_of: i16,
    veto: VetoSetup,
    bans_per_series: u8,
    mirror: bool,
    same_team: bool,
    no_repeat: Option<NoRepeatWho>,
    who_starts: VetoRoomStart,
    who_acts: VetoRoomActors,
}

async fn settings(conn: &mut PgConnection, body: NewVetoRoom) -> Result<Settings, ApiError> {
    if !(1..=MAX_TEAM_SIZE).contains(&body.team_size) {
        return Err(ApiError::invalid("tournament.team_size_invalid")
            .with_params([("min", 1), ("max", i64::from(MAX_TEAM_SIZE))]));
    }
    if !BEST_OF.contains(&body.best_of) {
        return Err(ApiError::invalid("format.invalid_best_of")
            .with_params([("best_of", i64::from(body.best_of))]));
    }
    let mut veto = resolve_veto(conn, body.veto.unwrap_or_default(), None).await?;
    match veto.map_mode {
        MapMode::Veto => {}
        MapMode::Off if veto.champion_phase => veto.pool.clear(),
        MapMode::Off => return Err(ApiError::invalid("veto_room.no_phases")),
        MapMode::Vote | MapMode::Random => {
            return Err(ApiError::invalid("veto_room.map_mode_invalid"));
        }
    }
    let longest = u8::try_from(body.best_of).unwrap_or(u8::MAX);
    veto.check(longest)?;
    let rules = body.champion_rules.unwrap_or_default();
    let max_bans = engine::champions::MAX_BANS_PER_SERIES;
    if u32::from(rules.bans_per_series) > max_bans {
        return Err(ApiError::invalid("tournament.bans_invalid")
            .with_params([("max", i64::from(max_bans))]));
    }
    let same_team = rules.same_team && body.team_size > 1;
    engine::veto::PickOrder::from(veto.pick_order).check(rules.mirror)?;
    if veto.champion_phase {
        let rule = rules.no_repeat.map(|who| NoRepeatRule {
            who: match who {
                NoRepeatWho::Player => engine::champions::NoRepeatWho::Player,
                NoRepeatWho::Team => engine::champions::NoRepeatWho::Team,
            },
            scope: NoRepeatScope::Series,
        });
        let budget = ChampionBudget {
            pool_size: champion_pool(&mut *conn).await?,
            bans_per_series: u32::from(rules.bans_per_series),
            team_size: u32::try_from(body.team_size).unwrap_or(1),
            mirror: rules.mirror,
            same_team,
            games_in_scope: u32::from(longest),
        };
        check_budget(rule, budget)?;
    }
    Ok(Settings {
        team_size: body.team_size,
        best_of: body.best_of,
        veto,
        bans_per_series: rules.bans_per_series,
        mirror: rules.mirror,
        same_team,
        no_repeat: rules.no_repeat,
        who_starts: body.who_starts,
        // No duelo, um lugar por lado: o mesmo nos dois.
        who_acts: if body.team_size > 1 {
            body.who_acts
        } else {
            VetoRoomActors::All
        },
    })
}
