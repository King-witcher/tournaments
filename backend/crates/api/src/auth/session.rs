//! Cookies da sessão e do state do OAuth, e a configuração que eles usam.

use axum::{
    http::{HeaderMap, header},
    response::Redirect,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

pub const SESSION_COOKIE: &str = "session";
pub const STATE_COOKIE: &str = "oauth_state";
pub const SESSION_DAYS: i64 = 30;

/// Configuração de auth que os handlers precisam em tempo de execução.
#[derive(Clone, Debug)]
pub struct AuthSettings {
    /// `None` desliga o login com Discord (503).
    pub discord_client_id: Option<String>,
    pub redirect_uri: String,
    /// Origem do front, sem barra final.
    pub web_origin: String,
    pub cookie_domain: Option<String>,
    pub cookie_secure: bool,
}

impl AuthSettings {
    pub fn session_cookie(&self, value: String) -> Cookie<'static> {
        let mut cookie = Cookie::build((SESSION_COOKIE, value))
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
        let mut cookie = Cookie::build((SESSION_COOKIE, "")).path("/");
        if let Some(domain) = &self.cookie_domain {
            cookie = cookie.domain(domain.clone());
        }
        cookie.build()
    }

    pub fn state_cookie(&self, value: String) -> Cookie<'static> {
        Cookie::build((STATE_COOKIE, value))
            .http_only(true)
            .same_site(SameSite::Lax)
            .path("/auth")
            .secure(self.cookie_secure)
            .max_age(time::Duration::seconds(600))
            .build()
    }

    pub fn clear_state_cookie(&self) -> Cookie<'static> {
        Cookie::build((STATE_COOKIE, "")).path("/auth").build()
    }

    pub fn web_redirect(&self, path: &str) -> Redirect {
        Redirect::to(&format!("{}{}", self.web_origin, path))
    }
}

/// Para que o OAuth do Discord foi aberto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OAuthIntent {
    /// Entrar (ou criar a conta) com o Discord.
    Login,
    /// Ligar o Discord à conta logada.
    Link,
}

/// O que o cookie `oauth_state` guarda entre o redirect e o callback.
#[derive(Debug, PartialEq, Eq)]
pub struct OAuthFlow {
    pub state: String,
    pub intent: OAuthIntent,
    pub return_to: String,
}

impl OAuthFlow {
    /// `{state}.{login|link}.{return_to em base64url}`. Nem o state nem o
    /// base64url têm '.', então ele separa os campos.
    pub fn encode(&self) -> String {
        let intent = match self.intent {
            OAuthIntent::Login => "login",
            OAuthIntent::Link => "link",
        };
        let return_to = URL_SAFE_NO_PAD.encode(&self.return_to);
        format!("{}.{intent}.{return_to}", self.state)
    }

    /// Lê o cookie; `return_to` passa de novo por [`sanitize_return_to`].
    pub fn decode(value: &str) -> Option<Self> {
        let mut parts = value.split('.');
        let (Some(state), Some(intent), Some(return_to), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return None;
        };
        let intent = match intent {
            "login" => OAuthIntent::Login,
            "link" => OAuthIntent::Link,
            _ => return None,
        };
        let return_to = URL_SAFE_NO_PAD
            .decode(return_to)
            .ok()
            .and_then(|b| String::from_utf8(b).ok());
        Some(Self {
            state: state.to_owned(),
            intent,
            return_to: sanitize_return_to(return_to.as_deref()),
        })
    }
}

/// Só caminhos internos; qualquer outra coisa vira `/` (evita open redirect).
pub fn sanitize_return_to(raw: Option<&str>) -> String {
    match raw {
        Some(p)
            if p.starts_with('/')
                && !p.starts_with("//")
                && !p.starts_with("/\\")
                && !p.chars().any(char::is_control) =>
        {
            p.to_owned()
        }
        _ => "/".to_owned(),
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

/// `path` com `auth_error=<código>` na query, para o front mostrar o erro.
pub fn with_auth_error(path: &str, code: &str) -> String {
    let separator = if path.contains('?') { '&' } else { '?' };
    format!("{path}{separator}auth_error={code}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oauth_flow_round_trips() {
        for intent in [OAuthIntent::Login, OAuthIntent::Link] {
            let flow = OAuthFlow {
                state: "abc_-123".into(),
                intent,
                return_to: "/pt/account?x=1.2".into(),
            };
            assert_eq!(OAuthFlow::decode(&flow.encode()), Some(flow));
        }
    }

    #[test]
    fn oauth_flow_rejects_bad_cookies() {
        let return_to = URL_SAFE_NO_PAD.encode("/pt");
        for bad in [
            String::new(),
            "abc".into(),
            format!("abc.{return_to}"),
            format!("abc.other.{return_to}"),
            format!("abc.login.{return_to}.extra"),
        ] {
            assert_eq!(OAuthFlow::decode(&bad), None, "{bad}");
        }
        let evil = format!("abc.link.{}", URL_SAFE_NO_PAD.encode("//evil.com"));
        assert_eq!(OAuthFlow::decode(&evil).unwrap().return_to, "/");
    }

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
    fn auth_error_goes_in_the_query() {
        assert_eq!(with_auth_error("/", "x.y"), "/?auth_error=x.y");
        assert_eq!(with_auth_error("/pt?a=1", "x.y"), "/pt?a=1&auth_error=x.y");
    }
}
