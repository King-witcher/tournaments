//! O envio ao Discord: um POST por mensagem para o webhook. A base é
//! `https://discord.com`; os testes apontam para um servidor de mentira
//! (`with_base`). O endereço do webhook leva a credencial, então nenhum erro
//! daqui o carrega (`without_url`) e nada dele vai para o log.

use std::{sync::Arc, time::Duration};

use reqwest::{StatusCode, header::HeaderMap};
use serde_json::Value;
use tokio::sync::Notify;

const BASE: &str = "https://discord.com";
/// Tempo máximo de uma ida ao Discord.
const TIMEOUT: Duration = Duration::from_secs(10);
/// Espera quando o Discord manda esperar sem dizer quanto.
const DEFAULT_WAIT: Duration = Duration::from_secs(5);
/// Teto da espera que o Discord pede: um valor absurdo não trava o webhook.
const MAX_WAIT: Duration = Duration::from_secs(10 * 60);

/// O que o Discord respondeu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// 2xx. `wait`: o ritmo acabou (`X-RateLimit-Remaining: 0`) e a próxima
    /// mensagem espera esse tempo.
    Sent { wait: Option<Duration> },
    /// 429: tentar de novo depois de `retry_after`.
    RateLimited { retry_after: Duration },
    /// 401 ou 404: o webhook foi apagado (ou o token mudou).
    Gone,
    /// Outro 4xx: a mensagem não serve e não adianta repetir.
    Rejected(StatusCode),
    /// 5xx ou falha de rede: pode servir mais tarde. Sem endereço nem token.
    Failed(String),
}

/// O cliente HTTP dos webhooks, com o despertador da fila (`wake`).
#[derive(Clone)]
pub struct Webhooks {
    client: reqwest::Client,
    base: Arc<str>,
    wake: Arc<Notify>,
}

impl Webhooks {
    pub fn new() -> Self {
        Self::with_base(BASE)
    }

    /// Para os testes: a base no lugar de `https://discord.com`.
    pub fn with_base(base: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("tournaments-api")
                .timeout(TIMEOUT)
                // Um redirecionamento levaria o token a outro endereço.
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .expect("cliente HTTP"),
            base: base.trim_end_matches('/').into(),
            wake: Arc::default(),
        }
    }

    /// Acorda quem esvazia a fila.
    pub fn wake(&self) {
        self.wake.notify_one();
    }

    pub(super) async fn woken(&self, timeout: Duration) {
        let _ = tokio::time::timeout(timeout, self.wake.notified()).await;
    }

    /// Manda `body` (o JSON de "executar webhook") ao webhook `id`/`token`.
    pub async fn post(&self, id: &str, token: &str, body: &Value) -> Outcome {
        let url = format!("{}/api/webhooks/{id}/{token}", self.base);
        let res = match self.client.post(url).json(body).send().await {
            Ok(res) => res,
            Err(e) => return Outcome::Failed(e.without_url().to_string()),
        };
        let status = res.status();
        let headers = res.headers().clone();
        if status.is_success() {
            return Outcome::Sent {
                wait: exhausted(&headers),
            };
        }
        match status {
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after = retry_after(&headers, res.json().await.ok());
                Outcome::RateLimited { retry_after }
            }
            StatusCode::UNAUTHORIZED | StatusCode::NOT_FOUND => Outcome::Gone,
            s if s.is_server_error() => Outcome::Failed(format!("o Discord respondeu {s}")),
            s => Outcome::Rejected(s),
        }
    }
}

impl Default for Webhooks {
    fn default() -> Self {
        Self::new()
    }
}

/// Quanto esperar quando a janela do webhook acabou (~5 mensagens por 2 s):
/// `X-RateLimit-Remaining: 0` com `X-RateLimit-Reset-After`.
fn exhausted(headers: &HeaderMap) -> Option<Duration> {
    let number = |name: &str| headers.get(name)?.to_str().ok()?.parse::<f64>().ok();
    (number("x-ratelimit-remaining")? <= 0.0)
        .then(|| seconds(number("x-ratelimit-reset-after").unwrap_or(1.0)))
}

/// A espera de um 429: `retry_after` do corpo (segundos, com fração) ou o
/// cabeçalho `Retry-After`.
fn retry_after(headers: &HeaderMap, body: Option<Value>) -> Duration {
    let from_body = body.and_then(|b| b.get("retry_after")?.as_f64());
    let from_header = || {
        headers
            .get("retry-after")?
            .to_str()
            .ok()?
            .parse::<f64>()
            .ok()
    };
    from_body.or_else(from_header).map_or(DEFAULT_WAIT, seconds)
}

fn seconds(value: f64) -> Duration {
    if value.is_finite() && value > 0.0 {
        Duration::from_secs_f64(value).min(MAX_WAIT)
    } else {
        DEFAULT_WAIT
    }
}

#[cfg(test)]
mod tests {
    use reqwest::header::HeaderValue;
    use serde_json::json;

    use super::*;

    fn headers(pairs: &[(&'static str, &'static str)]) -> HeaderMap {
        pairs
            .iter()
            .map(|(k, v)| (k.parse().unwrap(), HeaderValue::from_static(v)))
            .collect()
    }

    #[test]
    fn waits_only_when_the_window_is_spent() {
        assert_eq!(exhausted(&headers(&[])), None);
        let left = headers(&[("x-ratelimit-remaining", "2")]);
        assert_eq!(exhausted(&left), None);
        let spent = headers(&[
            ("x-ratelimit-remaining", "0"),
            ("x-ratelimit-reset-after", "1.5"),
        ]);
        assert_eq!(exhausted(&spent), Some(Duration::from_millis(1500)));
    }

    #[test]
    fn retry_after_comes_from_the_body_then_the_header() {
        let none = headers(&[]);
        let body = Some(json!({ "retry_after": 0.75 }));
        assert_eq!(retry_after(&none, body), Duration::from_millis(750));
        let header = headers(&[("retry-after", "3")]);
        assert_eq!(retry_after(&header, None), Duration::from_secs(3));
        assert_eq!(retry_after(&none, None), DEFAULT_WAIT);
        let huge = Some(json!({ "retry_after": 1e12 }));
        assert_eq!(retry_after(&none, huge), MAX_WAIT);
    }
}
