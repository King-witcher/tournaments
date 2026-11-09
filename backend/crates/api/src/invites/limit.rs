//! Limite por IP da prévia e do aceite (`/invites/{token}`): com códigos de 8
//! caracteres, dá para tentar adivinhar. Fica em memória (`AppState::limits`),
//! sem Redis, e não olha quem tem conta: a prévia é pública.
//!
//! A página `/invite/{token}` busca a prévia no servidor do Next, que sai pelos
//! IPs da Vercel: pelo `ClientIp` todos cairiam no mesmo balde. Ele manda o IP
//! de quem abriu a página em `x-viewer-ip` (`getInvite`, no front), e só vale com
//! o segredo do Next (`Bans::trusted`), senão bastaria trocar o cabeçalho a cada
//! pedido.

use std::time::Duration;

use axum::http::{HeaderMap, StatusCode};

use crate::{
    AppState,
    shared::{ApiError, ClientIp},
};

/// Pedidos por IP na janela: abrir o link e aceitar são 2 ou 3, e uma LAN em
/// dia de torneio sai toda pelo mesmo IP.
const MAX_PER_IP: u32 = 120;
const WINDOW: Duration = Duration::from_secs(10 * 60);
const VIEWER_IP_HEADER: &str = "x-viewer-ip";

/// Conta um pedido: passou do teto → 429 `invite.rate_limited` `{minutes}`.
pub fn hit(state: &AppState, headers: &HeaderMap, ip: &ClientIp) -> Result<(), ApiError> {
    let Some(bucket) = viewer_bucket(state, headers, ip) else {
        return Ok(());
    };
    let key = format!("invite:ip:{bucket}");
    state.limits.hit(key, MAX_PER_IP, WINDOW).map_err(|wait| {
        tracing::warn!(bucket, ?wait, "limite de convites por IP");
        let minutes = i64::try_from(wait.as_secs() / 60 + 1).unwrap_or(i64::MAX);
        ApiError::new(StatusCode::TOO_MANY_REQUESTS, "invite.rate_limited")
            .with_params([("minutes", minutes)])
    })
}

/// De quem é o pedido. Do Next sem o cabeçalho (não deveria): `None`, porque
/// contar no IP dele somaria todo mundo, e o mesmo só sem proxy e sem o endereço
/// da conexão, no app dos testes.
fn viewer_bucket(state: &AppState, headers: &HeaderMap, ip: &ClientIp) -> Option<String> {
    if state.bans.trusted(headers) {
        let viewer = headers.get(VIEWER_IP_HEADER)?.to_str().ok()?;
        return ClientIp(Some(viewer.trim().parse().ok()?)).bucket();
    }
    ip.bucket()
}
