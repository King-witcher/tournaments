//! A resposta de quem está banido é a de uma rota que não existe, cabeçalho a
//! cabeçalho (o teste confere), e `/internal/bans` sem o segredo também: nada
//! distingue "banido" de "não tem nada aqui".

use std::net::IpAddr;

use axum::{
    Json, Router,
    body::to_bytes,
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tower::ServiceExt;
use tower_http::cors::CorsLayer;

use super::traps::{is_scanner, is_trap};
use crate::{
    AppState,
    auth::CurrentUser,
    shared::{ApiError, ClientIp},
};

/// O corpo que `/internal/bans` lê: um IP, nada mais.
const MAX_BODY: usize = 64;

/// Um router sem rotas, com o CORS de `cors`: a resposta de quem está banido sai
/// dele, igual à de uma rota que não existe sem repetir o que o CORS põe nela.
pub fn no_routes(cors: CorsLayer) -> Router {
    Router::new()
        .fallback(|| async { StatusCode::NOT_FOUND })
        .layer(cors)
}

/// Middleware: banido recebe 404 em tudo, menos o servidor do Next e quem tem
/// sessão válida. O scanner não tem sessão; quem tem é a pessoa que divide o IP
/// com ele (CGNAT), que não saberia por que o site sumiu. Fica por fora do CORS,
/// e o pedido vira `GET`, para o `OPTIONS` não ser atendido como preflight.
pub async fn refuse_banned(
    State((state, no_routes)): State<(AppState, Router)>,
    ip: ClientIp,
    req: Request,
    next: Next,
) -> Response {
    if !state.bans.is_banned(&ip) || state.bans.trusted(req.headers()) {
        return next.run(req).await;
    }
    let (mut parts, body) = req.into_parts();
    if CurrentUser::session_owner(&parts, &state).await.is_ok() {
        return next.run(Request::from_parts(parts, body)).await;
    }
    parts.method = Method::GET;
    let Ok(response) = no_routes.oneshot(Request::from_parts(parts, body)).await;
    response
}

/// Nenhuma rota atendeu: é aqui que se bane, e assim uma rota de verdade nunca é
/// uma armadilha, nem com um valor como `/tenants/foo.php`.
pub async fn unknown_route(
    State(state): State<AppState>,
    ip: ClientIp,
    uri: Uri,
    headers: HeaderMap,
) -> StatusCode {
    if is_trap(uri.path()) && is_scanner(&headers) && !state.bans.trusted(&headers) {
        state.bans.ban(&ip, uri.path());
    }
    StatusCode::NOT_FOUND
}

/// Só com o segredo. `GET`: a lista, que o proxy do front consulta. `POST` com um
/// IP: o front pegou um scanner. `DELETE` com um IP: tira o ban.
pub async fn internal_bans(State(state): State<AppState>, req: Request) -> Response {
    if !state.bans.trusted(req.headers()) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let method = req.method().clone();
    if method == Method::GET {
        return Json(state.bans.list()).into_response();
    }
    if method != Method::POST && method != Method::DELETE {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Some(ip) = body_ip(req).await else {
        return StatusCode::UNPROCESSABLE_ENTITY.into_response();
    };
    if method == Method::POST {
        state.bans.ban(&ClientIp(Some(ip)), "pego pelo front");
    } else if let Err(err) = state.bans.unban(&ClientIp(Some(ip))).await {
        return ApiError::from(err).into_response();
    }
    StatusCode::NO_CONTENT.into_response()
}

async fn body_ip(req: Request) -> Option<IpAddr> {
    let body = to_bytes(req.into_body(), MAX_BODY).await.ok()?;
    std::str::from_utf8(&body).ok()?.trim().parse().ok()
}
