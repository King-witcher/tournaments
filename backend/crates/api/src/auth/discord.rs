//! Login com o Discord (OAuth2 com PKCE, escopo `identify`): o code, com o
//! verifier, vira um access token, que lê o usuário. O avatar da conta vem
//! daqui.

use std::fmt;

use anyhow::Context;
use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Deserialize;

use super::session::OAuthClient;

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
}

impl HttpDiscord {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("telefrag-api")
                .build()
                .expect("cliente HTTP"),
            client_id,
            client_secret,
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
        let token: TokenResponse = checked(res)
            .await
            .context("Discord recusou o code")?
            .json()
            .await
            .context("resposta do token")?;
        Ok(token.access_token)
    }

    async fn fetch_user(&self, access_token: &str) -> anyhow::Result<DiscordUser> {
        let res = self
            .client
            .get("https://discord.com/api/users/@me")
            .bearer_auth(access_token)
            .send()
            .await
            .context("requisição do usuário")?;
        checked(res)
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
    body: String,
}

impl fmt::Display for RateLimited {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", StatusCode::TOO_MANY_REQUESTS, self.body)
    }
}

impl std::error::Error for RateLimited {}

/// A resposta, se deu certo; senão, o erro com o status e o começo do corpo
/// (o motivo do Discord, ou a página do Cloudflare num bloqueio), para o log.
async fn checked(res: reqwest::Response) -> anyhow::Result<reqwest::Response> {
    let status = res.status();
    if status.is_success() {
        return Ok(res);
    }
    let body: String = res
        .text()
        .await
        .unwrap_or_default()
        .chars()
        .take(300)
        .collect();
    if status == StatusCode::TOO_MANY_REQUESTS {
        return Err(RateLimited { body }.into());
    }
    Err(anyhow::anyhow!("{status}: {body}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(status: u16, body: &'static str) -> reqwest::Response {
        axum::http::Response::builder()
            .status(status)
            .body(body)
            .unwrap()
            .into()
    }

    #[tokio::test]
    async fn a_429_is_rate_limited_and_the_body_goes_to_the_log() {
        let err = checked(response(429, "error code: 1015"))
            .await
            .unwrap_err();
        assert!(err.root_cause().is::<RateLimited>());
        assert_eq!(err.to_string(), "429 Too Many Requests: error code: 1015");

        let err = checked(response(400, r#"{"error":"invalid_grant"}"#))
            .await
            .unwrap_err();
        assert!(!err.root_cause().is::<RateLimited>());
        assert!(err.to_string().contains("invalid_grant"));

        assert!(checked(response(200, "{}")).await.is_ok());
    }
}
