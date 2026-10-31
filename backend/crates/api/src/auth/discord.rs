//! Login com o Discord (OAuth2 com PKCE, escopo `identify`): o code, com o
//! verifier, vira um access token, que lê o usuário. O avatar da conta vem
//! daqui.

use anyhow::Context;
use async_trait::async_trait;
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
        let res: TokenResponse = self
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
            .context("requisição do token")?
            .error_for_status()
            .context("Discord recusou o code")?
            .json()
            .await
            .context("resposta do token")?;
        Ok(res.access_token)
    }

    async fn fetch_user(&self, access_token: &str) -> anyhow::Result<DiscordUser> {
        self.client
            .get("https://discord.com/api/users/@me")
            .bearer_auth(access_token)
            .send()
            .await
            .context("requisição do usuário")?
            .error_for_status()
            .context("Discord recusou o token")?
            .json()
            .await
            .context("resposta do usuário")
    }
}
