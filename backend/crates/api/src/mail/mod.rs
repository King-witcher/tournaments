//! Email: a forma canônica do endereço (`normalize`, que também recusa
//! domínios descartáveis e de apelidos), o envio atrás da trait `Mailer` e os
//! limites de envio no Redis. Não tem rota nem texto: quem manda (`auth`)
//! monta o email no idioma de quem recebe e chama `Mail::send`. Só depende
//! de `shared`.

mod address;
mod disposable;
mod kv;
mod sender;

use std::sync::Arc;

use axum::http::StatusCode;

use crate::shared::{ApiError, Redis};

pub use address::normalize;
pub use sender::{Email, LogMailer, Mailer, ResendMailer};

/// Espera entre dois envios para o mesmo endereço.
const COOLDOWN_SECS: u64 = 60;

/// Teto de envios por dia quando `EMAIL_DAILY_LIMIT` não diz: abaixo dos 100
/// do plano grátis do Resend.
pub const DEFAULT_DAILY_LIMIT: u32 = 90;

/// O envio da API: provedor e teto do dia. Clonar é barato.
#[derive(Clone)]
pub struct Mail {
    mailer: Arc<dyn Mailer>,
    daily_limit: u32,
}

impl Mail {
    pub fn new(mailer: Arc<dyn Mailer>, daily_limit: u32) -> Self {
        Self {
            mailer,
            daily_limit,
        }
    }

    /// Reserva um envio para `to` nos limites, antes de gerar o link que vai
    /// nele: assim um pedido recusado não troca o link que já foi. Envio
    /// recente para o mesmo endereço → 429 `email.too_soon` com
    /// `{"seconds"}`; teto do dia → 503 `email.unavailable`.
    pub async fn reserve(&self, redis: &Redis, to: &str) -> Result<Ticket, ApiError> {
        match kv::reserve(redis, to, COOLDOWN_SECS, self.daily_limit).await? {
            kv::Slot::Granted => Ok(Ticket {
                mailer: self.mailer.clone(),
                to: to.to_owned(),
            }),
            kv::Slot::TooSoon { seconds } => Err(ApiError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "email.too_soon",
            )
            .with_params([("seconds", seconds)])),
            kv::Slot::DailyLimit => {
                tracing::warn!("teto diário de emails atingido");
                Err(ApiError::new(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "email.unavailable",
                ))
            }
        }
    }

    /// Como `reserve`, mas fora dos limites devolve `None` em vez do erro,
    /// para a resposta não revelar se a conta existe (recuperação de senha,
    /// cadastro com email em uso): o email só não sai.
    pub async fn reserve_quietly(
        &self,
        redis: &Redis,
        to: &str,
    ) -> Result<Option<Ticket>, ApiError> {
        match self.reserve(redis, to).await {
            Ok(ticket) => Ok(Some(ticket)),
            Err(err) if err.status == StatusCode::INTERNAL_SERVER_ERROR => Err(err),
            Err(_) => Ok(None),
        }
    }
}

/// Um envio reservado para um endereço.
#[must_use = "a reserva gasta o limite; mande o email com `send`"]
pub struct Ticket {
    mailer: Arc<dyn Mailer>,
    to: String,
}

impl Ticket {
    /// Manda em segundo plano: quem pediu não espera o provedor, e a falha
    /// dele fica no log.
    pub fn send(self, email: Email) {
        debug_assert_eq!(
            email.to, self.to,
            "email para outro endereço que o reservado"
        );
        tokio::spawn(async move {
            if let Err(err) = self.mailer.send(&email).await {
                tracing::error!(error = ?err, subject = %email.subject, "envio de email falhou");
            }
        });
    }
}
