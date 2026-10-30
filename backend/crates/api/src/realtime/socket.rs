//! `GET /ws`: um WebSocket por aba, com a sessão do cookie. O `Origin` precisa
//! ser o do front: o navegador manda o cookie no upgrade de qualquer site, e
//! sem essa conferência outro site abriria o socket como quem o visita.

use std::time::Duration;

use axum::{
    extract::{
        FromRequestParts, State,
        ws::{
            CloseFrame, Message, WebSocket, WebSocketUpgrade, rejection::WebSocketUpgradeRejection,
        },
    },
    http::{StatusCode, header, request::Parts},
    response::Response,
};
use axum_extra::extract::CookieJar;
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use tokio::{
    sync::broadcast::error::RecvError,
    time::{Instant, interval_at},
};

use super::{
    connection::{Connection, Delivery},
    types::{ClientMessage, ServerMessage, Topic},
};
use crate::{
    AppState,
    auth::{CurrentUser, current_session_id},
    notifications, outbox,
    shared::{ApiError, ErrorBody},
    tenants::TenantMember,
    tournaments,
};

/// Intervalo do ping: proxy derruba conexão parada, e quem não responde a um
/// ping até o próximo sai.
const PING_EVERY: Duration = Duration::from_secs(30);
/// Fechamento quando a sessão acaba (sair, encerrar a sessão, trocar a
/// senha): o front não reconecta.
const SESSION_ENDED: u16 = 4001;

/// O `Origin` da requisição é o do front (`WEB_ORIGIN`). 403
/// `realtime.origin_forbidden` sem ele ou com outro.
pub struct SameOrigin;

impl FromRequestParts<AppState> for SameOrigin {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        let origin = parts
            .headers
            .get(header::ORIGIN)
            .and_then(|o| o.to_str().ok());
        if origin != Some(state.auth.web_origin.as_str()) {
            return Err(ApiError::forbidden("realtime.origin_forbidden"));
        }
        Ok(Self)
    }
}

#[utoipa::path(
    get,
    path = "/ws",
    tag = "realtime",
    responses(
        (status = 101, description = "WebSocket aberto. Mensagens em JSON: o navegador manda `ClientMessage` (assinar ou desfazer a assinatura de uma comunidade ou de um torneio) e recebe `ServerMessage` (a primeira é `unread`, quantas notificações faltam ver; depois, mudanças do que assinou e de \"Seus torneios\", avisos, acesso perdido, respostas). Ping a cada 30 s; quem não responde sai. Fechamento 4001: a sessão acabou (não reconecte). Em qualquer outra queda, reconecte, assine de novo e recarregue, porque o que mudou no meio se perdeu"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "realtime.origin_forbidden (o Origin não é o do site)"),
        (status = UPGRADE_REQUIRED, body = ErrorBody, description = "realtime.upgrade_required (não é um pedido de WebSocket)"),
    )
)]
pub async fn ws(
    _origin: SameOrigin,
    State(state): State<AppState>,
    user: CurrentUser,
    jar: CookieJar,
    upgrade: Result<WebSocketUpgrade, WebSocketUpgradeRejection>,
) -> Result<Response, ApiError> {
    let upgrade = upgrade
        .map_err(|_| ApiError::new(StatusCode::UPGRADE_REQUIRED, "realtime.upgrade_required"))?;
    let session = current_session_id(&jar).ok_or(ApiError::unauthenticated())?;
    Ok(upgrade.on_upgrade(move |socket| run(state, user, session, socket)))
}

type Sink = SplitSink<WebSocket, Message>;

async fn run(state: AppState, user: CurrentUser, session: String, socket: WebSocket) {
    let mut connection = Connection::new(user.id, session);
    let mut events = state.realtime.subscribe();
    let (mut sink, mut stream) = socket.split();
    // Depois de entrar no canal: um aviso que chega no meio pode contar duas
    // vezes (a mais, até abrir o sininho), mas não fica de fora.
    if send_unread(&state, &mut sink, user.id).await.is_err() {
        return;
    }
    let mut ping = interval_at(Instant::now() + PING_EVERY, PING_EVERY);
    let mut waiting_pong = false;
    loop {
        let sent = tokio::select! {
            incoming = stream.next() => {
                let Some(Ok(message)) = incoming else { break };
                waiting_pong = false;
                match message {
                    Message::Text(text) => {
                        let reply = handle(&state, &user, &mut connection, text.as_str()).await;
                        send(&mut sink, &reply).await
                    }
                    Message::Close(_) => break,
                    _ => Ok(()),
                }
            }
            event = events.recv() => match event {
                Ok(event) => match connection.deliver(&event) {
                    Delivery::Send(message) => send(&mut sink, &message).await,
                    Delivery::Skip => Ok(()),
                    Delivery::Close => {
                        let frame = CloseFrame {
                            code: SESSION_ENDED,
                            reason: "auth.session_ended".into(),
                        };
                        let _ = sink.send(Message::Close(Some(frame))).await;
                        break;
                    }
                },
                Err(RecvError::Lagged(_)) => {
                    // Os avisos perdidos também ficaram fora da contagem.
                    let mut result = send_unread(&state, &mut sink, user.id).await;
                    for message in connection.resync() {
                        result = send(&mut sink, &message).await;
                    }
                    result
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
        };
        if sent.is_err() {
            break;
        }
    }
}

async fn send(sink: &mut Sink, message: &ServerMessage) -> Result<(), axum::Error> {
    let text = serde_json::to_string(message).expect("a mensagem vira JSON");
    sink.send(Message::Text(text.into())).await
}

/// Quantas notificações faltam ver (`unread`). Se o banco falha, a conexão
/// segue sem a contagem, com log.
async fn send_unread(state: &AppState, sink: &mut Sink, user: i64) -> Result<(), axum::Error> {
    match notifications::unread_count(&state.db, user).await {
        Ok(count) => send(sink, &ServerMessage::Unread { count }).await,
        Err(e) => {
            tracing::error!(error = %e, "contagem das notificações falhou");
            Ok(())
        }
    }
}

/// Uma mensagem do navegador e a resposta.
async fn handle(
    state: &AppState,
    user: &CurrentUser,
    connection: &mut Connection,
    text: &str,
) -> ServerMessage {
    let Ok(message) = serde_json::from_str::<ClientMessage>(text) else {
        return ServerMessage::Error {
            code: "request.invalid_body".into(),
            tenant: None,
            tournament: None,
        };
    };
    match message {
        ClientMessage::Subscribe { tenant, tournament } => {
            let topic = Topic { tenant, tournament };
            // Ler o torneio pode abrir o check-in (e emitir): acorda o
            // despachante, como a camada das rotas.
            let (allowed, emitted) = outbox::track(check(state, user, &topic)).await;
            if emitted {
                state.realtime.wake();
            }
            let result = allowed.and_then(|()| {
                connection
                    .subscribe(topic.clone())
                    .then_some(())
                    .ok_or(ApiError::conflict("realtime.too_many_subscriptions"))
            });
            match result {
                Ok(()) => ServerMessage::Subscribed {
                    tenant: topic.tenant,
                    tournament: topic.tournament,
                },
                Err(e) => ServerMessage::Error {
                    code: e.code.into(),
                    tenant: Some(topic.tenant),
                    tournament: topic.tournament,
                },
            }
        }
        ClientMessage::Unsubscribe { tenant, tournament } => {
            let topic = Topic { tenant, tournament };
            connection.unsubscribe(&topic);
            ServerMessage::Unsubscribed {
                tenant: topic.tenant,
                tournament: topic.tournament,
            }
        }
    }
}

/// O acesso como na leitura: membro ativo da comunidade (404
/// `tenant.not_found`) e, no torneio, quem pode vê-lo (404
/// `tournament.not_found`).
async fn check(state: &AppState, user: &CurrentUser, topic: &Topic) -> Result<(), ApiError> {
    let member: TenantMember = TenantMember::of(&state.db, user.clone(), &topic.tenant).await?;
    if let Some(slug) = &topic.tournament {
        let mut conn = state.db.acquire().await?;
        tournaments::find_visible(&mut conn, &member, slug).await?;
    }
    Ok(())
}
