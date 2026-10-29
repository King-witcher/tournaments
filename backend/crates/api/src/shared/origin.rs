//! Pedido que muda algo (método que não é de leitura) vindo de uma página de
//! outra origem é recusado antes da rota, com 403 `request.cross_origin`. É a
//! defesa contra CSRF além do `SameSite=Lax` do cookie da sessão, que não
//! separa os subdomínios do mesmo site, e do CORS, que não barra o envio de
//! um formulário (só a leitura da resposta). O navegador manda `Origin` em
//! todo pedido assim; sem ele, quem pede não é um navegador (o servidor do
//! Next, o curl), não leva o cookie de outra pessoa e passa. Passam o front
//! (`WEB_ORIGIN`) e a própria API (o Scalar em `/docs`).

use axum::{
    extract::{Request, State},
    http::{HeaderValue, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

use super::ApiError;

/// Middleware: `web` é a origem do front, a mesma do CORS.
pub async fn refuse_cross_origin(
    State(web): State<HeaderValue>,
    req: Request,
    next: Next,
) -> Response {
    let allowed = req.method().is_safe()
        || match req.headers().get(header::ORIGIN) {
            None => true,
            Some(origin) => *origin == web || same_host(origin, req.headers().get(header::HOST)),
        };
    if allowed {
        next.run(req).await
    } else {
        ApiError::forbidden("request.cross_origin").into_response()
    }
}

/// `Origin` com o host (e a porta) do próprio pedido: uma página da API.
fn same_host(origin: &HeaderValue, host: Option<&HeaderValue>) -> bool {
    let authority = origin
        .to_str()
        .ok()
        .and_then(|origin| origin.split_once("://"))
        .map(|(_, authority)| authority);
    match (authority, host.and_then(|host| host.to_str().ok())) {
        (Some(authority), Some(host)) => authority.eq_ignore_ascii_case(host),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_host_compares_the_authority() {
        let value = HeaderValue::from_static;
        assert!(same_host(
            &value("http://localhost:4000"),
            Some(&value("localhost:4000"))
        ));
        assert!(same_host(
            &value("https://API.exemplo.com"),
            Some(&value("api.exemplo.com"))
        ));
        assert!(!same_host(
            &value("https://exemplo.com"),
            Some(&value("api.exemplo.com"))
        ));
        assert!(!same_host(
            &value("http://localhost:4001"),
            Some(&value("localhost:4000"))
        ));
        assert!(!same_host(&value("null"), Some(&value("localhost:4000"))));
        assert!(!same_host(&value("http://localhost:4000"), None));
    }
}
