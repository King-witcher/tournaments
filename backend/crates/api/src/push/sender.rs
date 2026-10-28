//! Envio do Web Push, atrás de uma trait para os testes não usarem a internet.
//! O `web-push-native` só cifra o corpo (RFC 8291) e assina o VAPID (RFC
//! 8292); o HTTP é o `reqwest`.

use std::time::Duration;

use anyhow::{Context, anyhow, bail};
use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use reqwest::{StatusCode, header::HeaderValue};
use web_push_native::{
    Auth, WebPushBuilder,
    jwt_simple::algorithms::{ECDSAP256PublicKeyLike, ES256KeyPair},
    p256,
};

use crate::shared::Locale;

/// Quanto o serviço de push guarda o aviso se o navegador estiver
/// desconectado (e a validade da assinatura VAPID). Depois disso o aviso já
/// não serve.
const TTL: Duration = Duration::from_secs(60 * 60);
/// Tempo máximo de uma ida ao serviço de push: o despachante não fica preso.
const TIMEOUT: Duration = Duration::from_secs(10);

/// Para onde mandar: a inscrição do navegador.
#[derive(Debug, Clone)]
pub struct Target {
    pub id: i64,
    pub endpoint: String,
    /// Chave pública do navegador, base64url sem padding.
    pub p256dh: String,
    /// Segredo de autenticação, base64url sem padding.
    pub auth: String,
    /// Idioma dos avisos desta inscrição (vai no corpo, para o service worker
    /// traduzir).
    pub locale: Locale,
}

/// O que o serviço de push respondeu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sent {
    Delivered,
    /// 404 ou 410: a inscrição não existe mais e sai do banco.
    Gone,
}

#[async_trait]
pub trait WebPush: Send + Sync {
    /// Chave pública VAPID (base64url), que o navegador usa ao se inscrever.
    fn public_key(&self) -> &str;
    /// Manda `payload` (JSON) cifrado para `to`, com urgência alta.
    async fn send(&self, to: &Target, payload: &[u8]) -> anyhow::Result<Sent>;
}

pub struct HttpWebPush {
    client: reqwest::Client,
    key: ES256KeyPair,
    public_key: String,
    subject: String,
}

impl HttpWebPush {
    /// Chaves VAPID em base64url (a pública, ponto P-256 sem compressão; a
    /// privada, o escalar de 32 bytes) e o contato (`mailto:` ou `https:`).
    /// Recusa par que não confere.
    pub fn new(public_key: &str, private_key: &str, subject: String) -> anyhow::Result<Self> {
        let private = decode(private_key).context("VAPID_PRIVATE_KEY não é base64url")?;
        let key = ES256KeyPair::from_bytes(&private)
            .map_err(|e| anyhow!("VAPID_PRIVATE_KEY inválida: {e}"))?;
        let derived = URL_SAFE_NO_PAD.encode(key.public_key().public_key().to_bytes_uncompressed());
        if derived != public_key.trim_end_matches('=') {
            bail!("VAPID_PUBLIC_KEY não é a chave pública de VAPID_PRIVATE_KEY");
        }
        Ok(Self {
            client: reqwest::Client::builder()
                .user_agent("tournaments-api")
                .timeout(TIMEOUT)
                .build()
                .context("cliente HTTP")?,
            key,
            public_key: derived,
            subject,
        })
    }
}

#[async_trait]
impl WebPush for HttpWebPush {
    fn public_key(&self) -> &str {
        &self.public_key
    }

    async fn send(&self, to: &Target, payload: &[u8]) -> anyhow::Result<Sent> {
        let ua_public = p256::PublicKey::from_sec1_bytes(&decode(&to.p256dh)?)
            .map_err(|_| anyhow!("p256dh inválida"))?;
        let auth = decode(&to.auth)?;
        if auth.len() != 16 {
            bail!("auth com {} bytes", auth.len());
        }
        let mut ua_auth = Auth::default();
        ua_auth.copy_from_slice(&auth);
        let request =
            WebPushBuilder::new(to.endpoint.parse().context("endpoint")?, ua_public, ua_auth)
                .with_valid_duration(TTL)
                .with_vapid(&self.key, &self.subject)
                .build(payload)?;
        let mut request = reqwest::Request::try_from(request)?;
        request
            .headers_mut()
            .insert("Urgency", HeaderValue::from_static("high"));
        let status = self.client.execute(request).await?.status();
        match status {
            StatusCode::NOT_FOUND | StatusCode::GONE => Ok(Sent::Gone),
            s if s.is_success() => Ok(Sent::Delivered),
            s => bail!("serviço de push respondeu {s}"),
        }
    }
}

/// base64url com ou sem padding.
pub fn decode(value: &str) -> anyhow::Result<Vec<u8>> {
    Ok(URL_SAFE_NO_PAD.decode(value.trim_end_matches('='))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Par do exemplo do `web-push-native` (não é segredo de ninguém).
    const PRIVATE: &str = "RS0WdYWWo1HajXg3NZR1olzCf31i-ZBGDkFyCs7j1jw";

    fn public_of(private: &str) -> String {
        let key = ES256KeyPair::from_bytes(&decode(private).unwrap()).unwrap();
        URL_SAFE_NO_PAD.encode(key.public_key().public_key().to_bytes_uncompressed())
    }

    #[test]
    fn the_key_pair_must_match() {
        let public = public_of(PRIVATE);
        assert_eq!(public.len(), 87);
        let push = HttpWebPush::new(&public, PRIVATE, "mailto:a@b.c".into()).unwrap();
        assert_eq!(push.public_key(), public);
        let other = public_of("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAE");
        assert!(HttpWebPush::new(&other, PRIVATE, "mailto:a@b.c".into()).is_err());
        assert!(HttpWebPush::new(&public, "não", "mailto:a@b.c".into()).is_err());
    }
}
