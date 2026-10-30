//! O IP de quem fez a requisição, para os limites por IP. No Render a conexão
//! chega do proxy dele, que fica atrás do Cloudflare, e o `X-Forwarded-For`
//! não serve: o proxy só acrescenta à lista que o cliente mandou, e o último
//! item é o servidor do Cloudflare, o mesmo para muita gente. O IP vem do
//! `CF-Connecting-IP`, que o Cloudflare grava no lugar do que o cliente
//! mandar; sem ele (local, sem proxy), do endereço da conexão. Se a API sair
//! de trás do Cloudflare, o cabeçalho passa a ser forjável e precisa mudar.

use std::{
    convert::Infallible,
    net::{IpAddr, SocketAddr},
};

use axum::{
    extract::{ConnectInfo, FromRequestParts},
    http::request::Parts,
};

/// O cabeçalho com o IP de quem fala com o Cloudflare.
const HEADER: &str = "cf-connecting-ip";

/// O IP de quem pede. `None` só sem o cabeçalho e sem o endereço da conexão,
/// o que acontece no app dos testes, que não passa por socket.
pub struct ClientIp(pub Option<IpAddr>);

impl<S: Send + Sync> FromRequestParts<S> for ClientIp {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let forwarded = parts
            .headers
            .get(HEADER)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.trim().parse().ok());
        let connection = || {
            parts
                .extensions
                .get::<ConnectInfo<SocketAddr>>()
                .map(|ConnectInfo(addr)| addr.ip())
        };
        Ok(Self(forwarded.or_else(connection)))
    }
}

impl ClientIp {
    /// Quem divide o limite: o IPv4 inteiro; o IPv6 pelo /64, o bloco que o
    /// provedor dá a um cliente só (cada aparelho da casa troca o resto).
    pub fn bucket(&self) -> Option<String> {
        self.0.map(|ip| match ip.to_canonical() {
            IpAddr::V4(v4) => v4.to_string(),
            IpAddr::V6(v6) => {
                let [a, b, c, d, ..] = v6.segments();
                format!("{a:x}:{b:x}:{c:x}:{d:x}::/64")
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use axum::http::Request;

    use super::*;

    async fn extract(header: Option<&str>, peer: Option<&str>) -> ClientIp {
        let mut req = Request::builder();
        if let Some(value) = header {
            req = req.header(HEADER, value);
        }
        let (mut parts, ()) = req.body(()).unwrap().into_parts();
        if let Some(peer) = peer {
            parts
                .extensions
                .insert(ConnectInfo::<SocketAddr>(peer.parse().unwrap()));
        }
        ClientIp::from_request_parts(&mut parts, &()).await.unwrap()
    }

    #[tokio::test]
    async fn header_wins_over_the_connection() {
        let ip = extract(Some(" 203.0.113.7 "), Some("10.0.0.1:5000")).await;
        assert_eq!(ip.bucket().as_deref(), Some("203.0.113.7"));
        // Sem o cabeçalho (ou com lixo nele), o endereço da conexão.
        let ip = extract(Some("não é ip"), Some("10.0.0.1:5000")).await;
        assert_eq!(ip.bucket().as_deref(), Some("10.0.0.1"));
        assert_eq!(extract(None, None).await.bucket(), None);
    }

    #[tokio::test]
    async fn ipv6_counts_by_its_64_block() {
        let a = extract(Some("2001:db8:1:2:aaaa::1"), None).await;
        let b = extract(Some("2001:db8:1:2:bbbb::9"), None).await;
        assert_eq!(a.bucket().as_deref(), Some("2001:db8:1:2::/64"));
        assert_eq!(a.bucket(), b.bucket());
        // IPv4 escrito como IPv6 conta como o IPv4.
        let mapped = extract(Some("::ffff:203.0.113.7"), None).await;
        assert_eq!(mapped.bucket().as_deref(), Some("203.0.113.7"));
    }
}
