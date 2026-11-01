//! O cookie da sessão e a configuração de auth (provedores, origem do front,
//! atributos do cookie).

use axum::http::{HeaderMap, header};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};

use super::provider::Provider;
use crate::shared::token;

pub const SESSION_DAYS: i64 = 30;

/// Caracteres do hash (em hexadecimal) que identificam a sessão na lista e
/// nos eventos do tempo real: 64 bits, que não se repetem entre as sessões de
/// uma conta.
pub const SESSION_ID_LEN: usize = 16;

/// O id público da sessão de hash `hash` (em hexadecimal).
pub fn session_id(hash: &str) -> String {
    hash[..SESSION_ID_LEN].to_owned()
}

/// O app registrado num provedor: o que vai na autorização e na troca do
/// code. O secret fica no cliente HTTP do provedor.
#[derive(Clone, Debug)]
pub struct OAuthClient {
    pub client_id: String,
    /// A página de callback do front, `{WEB_ORIGIN}/auth/{provedor}/callback`
    /// (sem idioma: o proxy do front põe), que precisa estar registrada no
    /// provedor.
    pub redirect_uri: String,
}

/// Configuração de auth que os handlers precisam em tempo de execução.
#[derive(Clone, Debug)]
pub struct AuthSettings {
    /// `None` desliga o login com o Discord (503).
    pub discord: Option<OAuthClient>,
    /// `None` desliga o login com o Google (503).
    pub google: Option<OAuthClient>,
    /// Origem do front, sem barra final.
    pub web_origin: String,
    /// Nome do cookie da sessão (`COOKIE_NAME`, `session` por padrão).
    pub cookie_name: String,
    pub cookie_domain: Option<String>,
    pub cookie_secure: bool,
}

impl AuthSettings {
    pub fn client(&self, provider: Provider) -> Option<&OAuthClient> {
        match provider {
            Provider::Discord => self.discord.as_ref(),
            Provider::Google => self.google.as_ref(),
        }
    }

    /// O token da sessão, do cookie com o nome deste ambiente.
    pub fn session_token<'a>(&self, jar: &'a CookieJar) -> Option<&'a str> {
        jar.get(&self.cookie_name).map(|c| c.value())
    }

    /// Hash (em hexadecimal) da sessão do cookie.
    pub fn current_hash(&self, jar: &CookieJar) -> Option<String> {
        self.session_token(jar)
            .map(|t| token::hex(&token::hash_token(t)))
    }

    /// O id público da sessão do cookie.
    pub fn current_session_id(&self, jar: &CookieJar) -> Option<String> {
        self.current_hash(jar).map(|h| session_id(&h))
    }

    pub fn session_cookie(&self, value: String) -> Cookie<'static> {
        let mut cookie = Cookie::build((self.cookie_name.clone(), value))
            .http_only(true)
            .same_site(SameSite::Lax)
            .path("/")
            .secure(self.cookie_secure)
            .max_age(time::Duration::days(SESSION_DAYS));
        if let Some(domain) = &self.cookie_domain {
            cookie = cookie.domain(domain.clone());
        }
        cookie.build()
    }

    pub fn clear_session_cookie(&self) -> Cookie<'static> {
        let mut cookie = Cookie::build((self.cookie_name.clone(), "")).path("/");
        if let Some(domain) = &self.cookie_domain {
            cookie = cookie.domain(domain.clone());
        }
        cookie.build()
    }
}

/// Tamanho máximo do `User-Agent` guardado com a sessão.
const USER_AGENT_MAX: usize = 256;

/// O `User-Agent` do login, que a lista de sessões mostra para a pessoa
/// reconhecer cada uma.
pub fn user_agent(headers: &HeaderMap) -> Option<String> {
    let agent: String = headers
        .get(header::USER_AGENT)?
        .to_str()
        .ok()?
        .trim()
        .chars()
        .take(USER_AGENT_MAX)
        .collect();
    (!agent.is_empty()).then_some(agent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_agent_is_trimmed_and_capped() {
        let headers = |value: &str| {
            let mut h = HeaderMap::new();
            h.insert(header::USER_AGENT, value.parse().unwrap());
            h
        };
        assert_eq!(user_agent(&HeaderMap::new()), None);
        assert_eq!(user_agent(&headers("  ")), None);
        assert_eq!(
            user_agent(&headers(" Firefox/140 ")).as_deref(),
            Some("Firefox/140")
        );
        let long = "a".repeat(USER_AGENT_MAX + 10);
        assert_eq!(user_agent(&headers(&long)).unwrap().len(), USER_AGENT_MAX);
    }

    #[test]
    fn session_uses_the_cookie_of_this_environment() {
        let settings = AuthSettings {
            discord: None,
            google: None,
            web_origin: "https://staging.telefrag.com.br".into(),
            cookie_name: "session_staging".into(),
            cookie_domain: Some(".staging.telefrag.com.br".into()),
            cookie_secure: true,
        };
        // O cookie da produção (`Domain=.telefrag.com.br`) chega junto.
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            "session=prod; session_staging=staging".parse().unwrap(),
        );
        let jar = CookieJar::from_headers(&headers);
        assert_eq!(settings.session_token(&jar), Some("staging"));
        assert_eq!(
            settings.session_cookie("t".into()).name(),
            "session_staging"
        );
        assert_eq!(settings.clear_session_cookie().name(), "session_staging");
    }
}
