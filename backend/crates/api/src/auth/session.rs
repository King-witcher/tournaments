//! Cookies da sessão e do state do OAuth, e a configuração que eles usam.

use axum::response::Redirect;
use axum_extra::extract::cookie::{Cookie, SameSite};

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
