//! Envio de email, atrás da trait `Mailer` para os testes não usarem a
//! internet: `ResendMailer` (API HTTP do Resend; o Render free bloqueia as
//! portas de SMTP) e `LogMailer`, que só escreve no log (desenvolvimento, sem
//! chave).

use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use serde_json::json;

const RESEND_URL: &str = "https://api.resend.com/emails";

/// Quanto o pedido espera o Resend: o cadastro e a troca de email esperam o
/// envio para dizer se ele falhou.
const TIMEOUT: Duration = Duration::from_secs(10);

/// Um email pronto, com o texto no idioma de quem recebe.
#[derive(Debug, Clone)]
pub struct Email {
    /// Endereço na forma canônica (`address::normalize`).
    pub to: String,
    pub subject: String,
    pub text: String,
    pub html: String,
}

#[async_trait]
pub trait Mailer: Send + Sync {
    async fn send(&self, email: &Email) -> anyhow::Result<()>;
}

pub struct ResendMailer {
    client: reqwest::Client,
    api_key: String,
    /// Remetente, como `Nome <nao-responda@dominio>`, num domínio verificado
    /// no Resend.
    from: String,
}

impl ResendMailer {
    pub fn new(api_key: String, from: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(TIMEOUT)
                .build()
                .expect("cliente HTTP"),
            api_key,
            from,
        }
    }
}

#[async_trait]
impl Mailer for ResendMailer {
    async fn send(&self, email: &Email) -> anyhow::Result<()> {
        let res = self
            .client
            .post(RESEND_URL)
            .bearer_auth(&self.api_key)
            .header(reqwest::header::USER_AGENT, "telefrag-api")
            .json(&json!({
                "from": self.from,
                "to": [email.to],
                "subject": email.subject,
                "text": email.text,
                "html": email.html,
            }))
            .send()
            .await
            .context("Resend fora do ar")?;
        let status = res.status();
        if !status.is_success() {
            let body = res.text().await.unwrap_or_default();
            anyhow::bail!("Resend respondeu {status}: {body}");
        }
        Ok(())
    }
}

/// Sem provedor: o email (com o link) vai para o log. Só para
/// desenvolvimento; em produção a API não sobe sem o Resend.
pub struct LogMailer;

#[async_trait]
impl Mailer for LogMailer {
    async fn send(&self, email: &Email) -> anyhow::Result<()> {
        tracing::info!(to = %email.to, subject = %email.subject, "email (sem envio)\n{}", email.text);
        Ok(())
    }
}
