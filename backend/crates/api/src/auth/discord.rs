//! Login com o Discord (OAuth2 com PKCE, escopo `identify`): o code, com o
//! verifier, vira um access token, que lê o usuário. O avatar da conta vem
//! daqui.
//!
//! Toda chamada ao Discord, aqui e nos webhooks, passa pela mesma saída
//! (`discord_gate`): o IP é um só, e o Cloudflare do Discord o bloqueia inteiro
//! depois de 10.000 respostas 401, 403 ou 429 em 10 minutos
//! (`docs/decisions/auth.md`, "Discord bloqueando o Render").

use std::{fmt, time::Duration};

use anyhow::Context;
use async_trait::async_trait;
use reqwest::{StatusCode, header::HeaderMap};
use serde::Deserialize;
use serde_json::Value;

use super::session::OAuthClient;
use crate::shared::OutboundGate;

/// Tempo máximo de uma ida ao Discord.
const TIMEOUT: Duration = Duration::from_secs(10);
/// Duas chamadas por segundo dão no máximo 1.200 em 10 minutos, mesmo que todas
/// sejam recusadas.
const GATE_INTERVAL: Duration = Duration::from_millis(500);
/// Três logins de uma vez (cada um faz duas chamadas) não esperam.
const GATE_BURST: u32 = 6;
/// Até aqui a chamada espera a vez; depois, é recusada. Soma mais dez chamadas:
/// oito logins ao mesmo tempo ainda passam.
const GATE_MAX_WAIT: Duration = Duration::from_secs(5);
/// Pausa depois de um 429 que não diz quanto esperar.
const DEFAULT_PAUSE: Duration = Duration::from_secs(5 * 60);
/// Teto da pausa: um valor absurdo não derruba o login por horas, e testar o
/// bloqueio de 15 em 15 minutos não o prolonga.
const MAX_PAUSE: Duration = Duration::from_secs(15 * 60);

/// A saída para o Discord. Uma só por processo, dividida por login e webhooks.
pub fn discord_gate() -> OutboundGate {
    OutboundGate::new(GATE_INTERVAL, GATE_BURST, GATE_MAX_WAIT)
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
}

impl DiscordUser {
    /// Nome global ou, sem ele, o username.
    pub fn display_name(&self) -> &str {
        self.global_name.as_deref().unwrap_or(&self.username)
    }

    /// URL do avatar no CDN do Discord, se o usuário tem um.
    pub fn avatar_url(&self) -> Option<String> {
        self.avatar
            .as_ref()
            .map(|a| format!("https://cdn.discordapp.com/avatars/{}/{a}.png", self.id))
    }
}

/// URL de autorização do Discord, para ligar. Para entrar, o front monta a
/// mesma (`features/auth/lib/authorize-url.ts`): mudou aqui, mude lá.
pub fn authorize_url(client: &OAuthClient, state: &str, code_challenge: &str) -> reqwest::Url {
    let mut url = reqwest::Url::parse("https://discord.com/oauth2/authorize").expect("url fixa");
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", &client.client_id)
        .append_pair("scope", "identify")
        .append_pair("state", state)
        .append_pair("code_challenge", code_challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("redirect_uri", &client.redirect_uri);
    url
}

/// Acesso ao Discord, atrás de uma trait para os testes não usarem a internet.
#[async_trait]
pub trait DiscordApi: Send + Sync {
    /// Troca o `code` do OAuth2 por um access token. O Discord confere o
    /// `code_verifier` contra o challenge da autorização.
    async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
        redirect_uri: &str,
    ) -> anyhow::Result<String>;
    async fn fetch_user(&self, access_token: &str) -> anyhow::Result<DiscordUser>;
}

pub struct HttpDiscord {
    client: reqwest::Client,
    client_id: String,
    client_secret: String,
    gate: OutboundGate,
}

impl HttpDiscord {
    pub fn new(client_id: String, client_secret: String, gate: OutboundGate) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("telefrag-api")
                .timeout(TIMEOUT)
                .build()
                .expect("cliente HTTP"),
            client_id,
            client_secret,
            gate,
        }
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[async_trait]
impl DiscordApi for HttpDiscord {
    async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
        redirect_uri: &str,
    ) -> anyhow::Result<String> {
        self.gate.admit().await?;
        let res = self
            .client
            .post("https://discord.com/api/oauth2/token")
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("code_verifier", code_verifier),
                ("redirect_uri", redirect_uri),
            ])
            .send()
            .await
            .context("requisição do token")?;
        let token: TokenResponse = checked(res, &self.gate)
            .await
            .context("Discord recusou o code")?
            .json()
            .await
            .context("resposta do token")?;
        Ok(token.access_token)
    }

    async fn fetch_user(&self, access_token: &str) -> anyhow::Result<DiscordUser> {
        self.gate.admit().await?;
        let res = self
            .client
            .get("https://discord.com/api/users/@me")
            .bearer_auth(access_token)
            .send()
            .await
            .context("requisição do usuário")?;
        checked(res, &self.gate)
            .await
            .context("Discord recusou o token")?
            .json()
            .await
            .context("resposta do usuário")
    }
}

/// O Discord respondeu 429. Em produção, quase sempre é o IP de saída do
/// Render, dividido com outros serviços e bloqueado pelo Cloudflare do Discord
/// por abuso de alguém (`docs/decisions/auth.md`, "Discord bloqueando o
/// Render"). Fica como causa raiz do erro, para a rota responder outro código.
#[derive(Debug)]
pub struct RateLimited {
    limit: DiscordLimit,
}

impl fmt::Display for RateLimited {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} ({})",
            StatusCode::TOO_MANY_REQUESTS,
            self.limit.body,
            self.limit
        )
    }
}

impl std::error::Error for RateLimited {}

/// O que um 429 do Discord diz.
#[derive(Debug)]
pub struct DiscordLimit {
    /// `retry_after` do corpo (segundos, com fração) ou o cabeçalho
    /// `Retry-After`.
    pub retry_after: Option<Duration>,
    scope: Option<String>,
    global: bool,
    /// O corpo é o JSON da API do Discord, com `retry_after`. Sem ele, quem
    /// respondeu foi o Cloudflare (`error code: 1015`).
    from_api: bool,
    ray: Option<String>,
    /// O começo do corpo. Fora do `Display`: o log dos webhooks não o leva,
    /// porque uma página de erro poderia repetir o endereço.
    body: String,
}

impl DiscordLimit {
    pub async fn read(res: reqwest::Response) -> Self {
        let headers = res.headers().clone();
        let body = res.text().await.unwrap_or_default();
        Self::parse(&headers, &body)
    }

    fn parse(headers: &HeaderMap, body: &str) -> Self {
        let text = |name: &str| Some(headers.get(name)?.to_str().ok()?.to_owned());
        let json: Option<Value> = serde_json::from_str(body).ok();
        let from_body = json.as_ref().and_then(|b| b.get("retry_after")?.as_f64());
        let from_header = || text("retry-after")?.parse::<f64>().ok();
        let scope = text("x-ratelimit-scope");
        Self {
            retry_after: from_body
                .or_else(from_header)
                .and_then(|secs| Duration::try_from_secs_f64(secs).ok())
                .filter(|wait| !wait.is_zero()),
            global: headers.contains_key("x-ratelimit-global")
                || scope.as_deref() == Some("global")
                || json.as_ref().and_then(|b| b.get("global")?.as_bool()) == Some(true),
            scope,
            from_api: from_body.is_some(),
            ray: text("cf-ray"),
            body: body.chars().take(300).collect(),
        }
    }

    /// O limite vale para o IP inteiro, e não só para uma rota ou um webhook:
    /// bloqueio do Cloudflare ou limite global.
    pub fn blocks_ip(&self) -> bool {
        self.global || !self.from_api
    }

    /// Quanto parar de chamar o Discord.
    pub fn pause(&self) -> Duration {
        self.retry_after
            .map_or(DEFAULT_PAUSE, |wait| wait.min(MAX_PAUSE))
    }
}

impl fmt::Display for DiscordLimit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "retry_after={:?} scope={} global={} cloudflare={} cf-ray={}",
            self.retry_after,
            self.scope.as_deref().unwrap_or("-"),
            self.global,
            !self.from_api,
            self.ray.as_deref().unwrap_or("-"),
        )
    }
}

/// A resposta, se deu certo; senão, o erro com o status e o começo do corpo
/// (o motivo do Discord, ou a página do Cloudflare num bloqueio), para o log.
/// Um 429 pausa a saída, para o bloqueio não ser prolongado.
async fn checked(res: reqwest::Response, gate: &OutboundGate) -> anyhow::Result<reqwest::Response> {
    let status = res.status();
    if status.is_success() {
        return Ok(res);
    }
    if status == StatusCode::TOO_MANY_REQUESTS {
        let limit = DiscordLimit::read(res).await;
        gate.pause(limit.pause());
        return Err(RateLimited { limit }.into());
    }
    let body: String = res
        .text()
        .await
        .unwrap_or_default()
        .chars()
        .take(300)
        .collect();
    Err(anyhow::anyhow!("{status}: {body}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::shared::Refused;

    fn response(status: u16, body: &'static str) -> reqwest::Response {
        axum::http::Response::builder()
            .status(status)
            .header("cf-ray", "abc-GRU")
            .body(body)
            .unwrap()
            .into()
    }

    fn headers(pairs: &[(&'static str, &'static str)]) -> HeaderMap {
        pairs
            .iter()
            .map(|(k, v)| (k.parse().unwrap(), v.parse().unwrap()))
            .collect()
    }

    #[tokio::test]
    async fn a_429_pauses_the_gate_and_says_why_in_the_log() {
        let gate = discord_gate();
        let err = checked(response(429, "error code: 1015"), &gate)
            .await
            .unwrap_err();
        assert!(err.root_cause().is::<RateLimited>());
        assert_eq!(
            err.to_string(),
            "429 Too Many Requests: error code: 1015 \
             (retry_after=None scope=- global=false cloudflare=true cf-ray=abc-GRU)"
        );
        assert!(matches!(gate.admit().await, Err(Refused::Paused { .. })));
    }

    #[tokio::test]
    async fn other_answers_do_not_pause_the_gate() {
        let gate = discord_gate();
        let err = checked(response(400, r#"{"error":"invalid_grant"}"#), &gate)
            .await
            .unwrap_err();
        assert!(!err.root_cause().is::<RateLimited>());
        assert!(err.to_string().contains("invalid_grant"));

        assert!(checked(response(200, "{}"), &gate).await.is_ok());
        assert!(gate.admit().await.is_ok());
    }

    #[test]
    fn cloudflare_and_the_global_limit_block_the_ip_and_a_bucket_does_not() {
        let none = headers(&[]);
        let cloudflare = DiscordLimit::parse(&none, "error code: 1015");
        assert!(cloudflare.blocks_ip());
        assert_eq!(cloudflare.pause(), DEFAULT_PAUSE);

        let bucket = DiscordLimit::parse(&none, r#"{"retry_after":0.75,"global":false}"#);
        assert!(!bucket.blocks_ip());
        assert_eq!(bucket.pause(), Duration::from_millis(750));

        let global = DiscordLimit::parse(&none, r#"{"retry_after":2,"global":true}"#);
        assert!(global.blocks_ip());
        let scoped = headers(&[("x-ratelimit-scope", "global")]);
        assert!(DiscordLimit::parse(&scoped, r#"{"retry_after":2}"#).blocks_ip());
    }

    #[test]
    fn the_wait_comes_from_the_body_then_the_header_and_has_a_ceiling() {
        let header = headers(&[("retry-after", "3")]);
        let from_header = DiscordLimit::parse(&header, "error code: 1015");
        assert_eq!(from_header.retry_after, Some(Duration::from_secs(3)));
        let from_body = DiscordLimit::parse(&header, r#"{"retry_after":1}"#);
        assert_eq!(from_body.retry_after, Some(Duration::from_secs(1)));

        let huge = DiscordLimit::parse(&headers(&[]), r#"{"retry_after":1e12}"#);
        assert_eq!(huge.pause(), MAX_PAUSE);
        let absurd = DiscordLimit::parse(&headers(&[]), r#"{"retry_after":1e300}"#);
        assert_eq!(absurd.retry_after, None);
    }
}
