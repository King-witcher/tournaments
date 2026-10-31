//! Login com o Google (OpenID Connect, com PKCE): o code, com o verifier,
//! vira o id token, que diz quem entrou. Os escopos são só `openid profile`: o id da conta Google e o nome,
//! sem email (o da conta daqui é confirmado por link) e sem usar a foto.

use anyhow::{Context, ensure};
use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::Deserialize;

use super::{handlers::DISPLAY_NAME_MAX, session::OAuthClient};

/// Nome da conta nova quando o Google não manda nenhum.
const FALLBACK_NAME: &str = "Google";

#[derive(Debug, Clone)]
pub struct GoogleUser {
    /// O `sub`: não muda nem é reaproveitado, ao contrário do email.
    pub id: String,
    pub name: Option<String>,
}

impl GoogleUser {
    /// Nome exibido da conta nova: o do Google numa linha, cortado no limite
    /// do nome exibido (o Discord já tem o mesmo limite).
    pub fn display_name(&self) -> String {
        let name: String = self
            .name
            .as_deref()
            .unwrap_or_default()
            .chars()
            .filter(|c| !c.is_control())
            .collect();
        let cut: String = name.trim().chars().take(DISPLAY_NAME_MAX).collect();
        match cut.trim_end() {
            "" => FALLBACK_NAME.to_owned(),
            name => name.to_owned(),
        }
    }
}

/// URL de autorização do Google. `select_account` mostra a escolha da conta
/// mesmo com uma só logada, para quem vincula ver qual vai. Para entrar, o
/// front monta a mesma (`features/auth/lib/authorize-url.ts`): mudou aqui,
/// mude lá.
pub fn authorize_url(client: &OAuthClient, state: &str, code_challenge: &str) -> reqwest::Url {
    let mut url =
        reqwest::Url::parse("https://accounts.google.com/o/oauth2/v2/auth").expect("url fixa");
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", &client.client_id)
        .append_pair("scope", "openid profile")
        .append_pair("state", state)
        .append_pair("code_challenge", code_challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("redirect_uri", &client.redirect_uri)
        .append_pair("prompt", "select_account");
    url
}

/// Acesso ao Google, atrás de uma trait para os testes não usarem a internet.
#[async_trait]
pub trait GoogleApi: Send + Sync {
    /// Troca o `code` do OAuth2 pelo id token e lê dele quem entrou. O Google
    /// confere o `code_verifier` contra o challenge da autorização.
    async fn fetch_user(
        &self,
        code: &str,
        code_verifier: &str,
        redirect_uri: &str,
    ) -> anyhow::Result<GoogleUser>;
}

pub struct HttpGoogle {
    client: reqwest::Client,
    client_id: String,
    client_secret: String,
}

impl HttpGoogle {
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
    id_token: String,
}

#[async_trait]
impl GoogleApi for HttpGoogle {
    async fn fetch_user(
        &self,
        code: &str,
        code_verifier: &str,
        redirect_uri: &str,
    ) -> anyhow::Result<GoogleUser> {
        let res: TokenResponse = self
            .client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("code_verifier", code_verifier),
                ("redirect_uri", redirect_uri),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
            ])
            .send()
            .await
            .context("requisição do token")?
            .error_for_status()
            .context("Google recusou o code")?
            .json()
            .await
            .context("resposta do token")?;
        user_from_id_token(&res.id_token, &self.client_id)
    }
}

#[derive(Deserialize)]
struct Claims {
    iss: String,
    aud: String,
    sub: String,
    name: Option<String>,
    given_name: Option<String>,
}

/// Lê quem entrou no id token. A assinatura não é conferida: o token veio
/// direto do endpoint de token do Google, por HTTPS e com o client secret, o
/// caso em que o próprio Google dispensa a conferência. Emissor e público
/// ainda são conferidos.
fn user_from_id_token(token: &str, client_id: &str) -> anyhow::Result<GoogleUser> {
    let payload = token.split('.').nth(1).context("id token sem payload")?;
    let json = URL_SAFE_NO_PAD
        .decode(payload.trim_end_matches('='))
        .context("payload do id token")?;
    let claims: Claims = serde_json::from_slice(&json).context("claims do id token")?;
    ensure!(
        matches!(
            claims.iss.as_str(),
            "https://accounts.google.com" | "accounts.google.com"
        ),
        "id token de outro emissor: {}",
        claims.iss
    );
    ensure!(claims.aud == client_id, "id token de outro app");
    ensure!(!claims.sub.is_empty(), "id token sem sub");
    Ok(GoogleUser {
        id: claims.sub,
        name: claims.name.or(claims.given_name),
    })
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    /// Id token com `claims` no payload (cabeçalho e assinatura de enfeite).
    fn token(claims: Value) -> String {
        let payload = URL_SAFE_NO_PAD.encode(claims.to_string());
        format!("eyJhbGciOiJSUzI1NiJ9.{payload}.assinatura")
    }

    fn claims(name: Value) -> Value {
        json!({
            "iss": "https://accounts.google.com",
            "aud": "app-1",
            "sub": "1098",
            "name": name,
            "given_name": "Ana",
            "picture": "https://lh3.googleusercontent.com/a/x",
        })
    }

    #[test]
    fn reads_sub_and_name() {
        let user = user_from_id_token(&token(claims(json!("Ana Souza"))), "app-1").unwrap();
        assert_eq!(user.id, "1098");
        assert_eq!(user.display_name(), "Ana Souza");

        let user = user_from_id_token(&token(claims(Value::Null)), "app-1").unwrap();
        assert_eq!(user.display_name(), "Ana", "sem name, o given_name");
    }

    #[test]
    fn refuses_other_app_or_issuer() {
        assert!(user_from_id_token(&token(claims(json!("Ana"))), "app-2").is_err());
        let mut other = claims(json!("Ana"));
        other["iss"] = json!("https://evil.example");
        assert!(user_from_id_token(&token(other), "app-1").is_err());
        let mut no_sub = claims(json!("Ana"));
        no_sub["sub"] = json!("");
        assert!(user_from_id_token(&token(no_sub), "app-1").is_err());
        assert!(user_from_id_token("sem-pontos", "app-1").is_err());
    }

    #[test]
    fn display_name_fits_the_limit() {
        let user = |name: Option<&str>| GoogleUser {
            id: "1".into(),
            name: name.map(Into::into),
        };
        let long = format!("{} Silva", "Maria Fernanda".repeat(3));
        let cut = user(Some(&long)).display_name();
        assert_eq!(cut.chars().count(), DISPLAY_NAME_MAX);
        assert_eq!(
            user(Some("  Ana\n Souza \u{7}")).display_name(),
            "Ana Souza"
        );
        // Cortado num espaço, não termina com ele.
        let spaced = format!("{} b", "a".repeat(DISPLAY_NAME_MAX - 1));
        assert_eq!(
            user(Some(&spaced)).display_name(),
            "a".repeat(DISPLAY_NAME_MAX - 1)
        );
        assert_eq!(user(None).display_name(), FALLBACK_NAME);
        assert_eq!(user(Some(" \t ")).display_name(), FALLBACK_NAME);
    }
}
