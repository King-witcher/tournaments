//! `GET /veto-rooms/{token}/ws`: o socket de uma sala de veto, sem login. O
//! token do caminho diz a sala; quem está nela é avisado a cada mudança e pede a
//! sala de novo. Conta nos limites de conexão da sala (`veto_rooms::Live`). O
//! `Origin` é conferido como no `GET /ws`, para outro site não usar o socket de
//! quem o visita para contar nos limites.

use axum::{
    extract::{
        RawQuery, State,
        ws::{
            CloseFrame, Message, WebSocket, WebSocketUpgrade, rejection::WebSocketUpgradeRejection,
        },
    },
    http::StatusCode,
    response::Response,
};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use tokio::{
    sync::broadcast::error::RecvError,
    time::{Instant, interval_at, sleep},
};

use super::{
    socket::{PING_EVERY, SameOrigin, tab_connection},
    types::RoomMessage,
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, PathParams},
    veto_rooms::{self, RoomSocket, VetoRoomPath},
};

/// Fechamento quando a sala venceu: o front não reconecta.
const ROOM_GONE: u16 = 4004;

#[utoipa::path(
    get,
    path = "/veto-rooms/{token}/ws",
    tag = "realtime",
    params(
        ("token" = String, Path, description = "Um link da sala: de gestão, de espectador ou de um lugar"),
        ("connection" = Option<String>, Query, description = "Id que a aba dá à conexão, como em `GET /ws`: a ação que manda o mesmo id em `x-realtime-connection` não volta a esta conexão"),
    ),
    responses(
        (status = 101, description = "WebSocket aberto. O navegador recebe `RoomMessage` em JSON: `changed` a cada mudança da sala (peça a sala de novo; o dado não vem aqui). Ping a cada 30 s; quem não responde sai. Fechamento 4004: a sala venceu (não reconecte). Em qualquer outra queda, reconecte e recarregue"),
        (status = FORBIDDEN, body = ErrorBody, description = "realtime.origin_forbidden (o Origin não é o do site)"),
        (status = NOT_FOUND, body = ErrorBody, description = "veto_room.not_found (não existe ou venceu)"),
        (status = UPGRADE_REQUIRED, body = ErrorBody, description = "realtime.upgrade_required (não é um pedido de WebSocket)"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "veto_room.too_many_connections (a sala ou a API no teto de conexões: recarregue de tempos em tempos)"),
    )
)]
pub async fn veto_room_socket(
    _origin: SameOrigin,
    State(state): State<AppState>,
    PathParams(path): PathParams<VetoRoomPath>,
    RawQuery(query): RawQuery,
    upgrade: Result<WebSocketUpgrade, WebSocketUpgradeRejection>,
) -> Result<Response, ApiError> {
    let upgrade = upgrade
        .map_err(|_| ApiError::new(StatusCode::UPGRADE_REQUIRED, "realtime.upgrade_required"))?;
    let access = veto_rooms::access(&state.db, &path.token)
        .await?
        .ok_or(ApiError::not_found("veto_room.not_found"))?;
    let socket = state.veto_rooms.join(access.room, access.spectator)?;
    let id = query.as_deref().and_then(tab_connection);
    Ok(upgrade.on_upgrade(move |ws| run(state, path.token, access.expires_at, socket, id, ws)))
}

async fn run(
    state: AppState,
    token: String,
    mut expires_at: DateTime<Utc>,
    mut room: RoomSocket,
    id: Option<String>,
    socket: WebSocket,
) {
    let (mut sink, mut stream) = socket.split();
    let mut ping = interval_at(Instant::now() + PING_EVERY, PING_EVERY);
    let mut waiting_pong = false;
    loop {
        let wait = (expires_at - Utc::now()).to_std().unwrap_or_default();
        let sent = tokio::select! {
            incoming = stream.next() => {
                let Some(Ok(message)) = incoming else { break };
                waiting_pong = false;
                if matches!(message, Message::Close(_)) {
                    break;
                }
                Ok(())
            }
            change = room.changes.recv() => match change {
                Ok(change) if change.origin.is_some() && change.origin == id => Ok(()),
                Ok(_) | Err(RecvError::Lagged(_)) => {
                    let text = serde_json::to_string(&RoomMessage::Changed)
                        .expect("a mensagem vira JSON");
                    sink.send(Message::Text(text.into())).await
                }
                Err(RecvError::Closed) => break,
            },
            _ = ping.tick() => {
                if waiting_pong {
                    break;
                }
                waiting_pong = true;
                sink.send(Message::Ping(Default::default())).await
            }
            () = sleep(wait) => {
                // A validade pode ter sido estendida quando a sala começou.
                match veto_rooms::access(&state.db, &token).await {
                    Ok(Some(access)) => {
                        expires_at = access.expires_at;
                        Ok(())
                    }
                    Ok(None) => {
                        let frame = CloseFrame {
                            code: ROOM_GONE,
                            reason: "veto_room.not_found".into(),
                        };
                        let _ = sink.send(Message::Close(Some(frame))).await;
                        break;
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "validade da sala de veto");
                        break;
                    }
                }
            }
        };
        if sent.is_err() {
            break;
        }
    }
}
