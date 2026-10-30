//! Email: o endereço na forma que é gravada (`Mail::normalize`: a canônica,
//! sem descartáveis nem apelidos, ou, com `EMAIL_STRICT` desligado, como foi
//! escrito), o layout no visual do site (`compose`), o envio atrás da trait
//! `Mailer` e os limites de envio no Redis. Não tem rota, e do texto só o nome
//! do site e o rodapé: quem manda (`auth`) escreve o email no idioma de quem
//! recebe, reserva o envio e chama `Ticket::send`. Só depende de `shared`.

mod address;
mod disposable;
mod kv;
mod layout;
mod sender;

use std::sync::Arc;

use axum::http::StatusCode;

use crate::shared::{ApiError, Redis};

pub use layout::{Action, Alert, Content, compose};
pub use sender::{Email, LogMailer, Mailer, ResendMailer};

/// Espera entre dois envios para o mesmo endereço.
const COOLDOWN_SECS: u64 = 60;

/// Teto de envios por dia quando `EMAIL_DAILY_LIMIT` não diz: abaixo dos 100
/// do plano grátis do Resend.
pub const DEFAULT_DAILY_LIMIT: u32 = 90;

/// O envio da API: provedor, teto do dia e se o endereço vai para a forma
/// canônica (`EMAIL_STRICT`). Clonar é barato.
#[derive(Clone)]
pub struct Mail {
    mailer: Arc<dyn Mailer>,
    daily_limit: u32,
    strict: bool,
}

impl Mail {
    pub fn new(mailer: Arc<dyn Mailer>, daily_limit: u32, strict: bool) -> Self {
        Self {
            mailer,
            daily_limit,
            strict,
        }
    }

    /// O endereço na forma que é gravada (`address::normalize`): com
    /// `EMAIL_STRICT` (o padrão), a canônica, sem descartáveis; sem ele (dev e
    /// staging), como foi escrito, em minúsculas.
    pub fn normalize(&self, raw: &str) -> Result<String, ApiError> {
        address::normalize(raw, self.strict)
    }

    /// Reserva um envio para `to` nos limites, antes de gerar o link que vai
    /// nele: assim um pedido recusado não troca o link que já foi. Envio
    /// recente para o mesmo endereço → 429 `email.too_soon` com
    /// `{"seconds"}`; teto do dia → 503 `email.unavailable`.
    pub async fn reserve(&self, redis: &Redis, to: &str) -> Result<Ticket, ApiError> {
        match kv::reserve(redis, to, COOLDOWN_SECS, self.daily_limit).await? {
            kv::Slot::Granted { day } => Ok(self.ticket(redis, to, day, true)),
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

    /// Reserva um aviso (o que mudou na conta) para `to`: conta no teto do
    /// dia, mas não espera nem marca a espera entre envios, que é dos links.
    /// Assim o aviso logo depois de um link (a senha trocada pelo link de
    /// recuperação) não se perde, e não atrasa o próximo link. Fora do teto,
    /// `None`: o aviso só não sai.
    pub async fn reserve_notice(
        &self,
        redis: &Redis,
        to: &str,
    ) -> Result<Option<Ticket>, ApiError> {
        match kv::reserve_today(redis, self.daily_limit).await? {
            kv::Slot::Granted { day } => Ok(Some(self.ticket(redis, to, day, false))),
            _ => {
                tracing::warn!("teto diário de emails atingido; aviso não enviado");
                Ok(None)
            }
        }
    }

    fn ticket(&self, redis: &Redis, to: &str, day: String, cooldown: bool) -> Ticket {
        Ticket {
            mailer: self.mailer.clone(),
            redis: redis.clone(),
            to: to.to_owned(),
            day,
            cooldown,
        }
    }
}

/// Um envio reservado para um endereço. Quem pediu e espera a resposta (o
/// link do cadastro, da troca de email) manda com `send`, que espera o
/// provedor; o que ninguém espera na tela vai em segundo plano, por
/// `send_in_background` (exclusão agendada) ou por `send` dentro da própria
/// tarefa de quem manda (avisos, recuperação de senha, que desfaz o link). Nos dois, a falha do provedor
/// vai para o log e devolve a reserva: a espera de 60 s sai e o envio não
/// conta no teto do dia, então dá para pedir de novo na hora.
#[must_use = "a reserva gasta o limite; mande o email com `send` ou `send_in_background`"]
pub struct Ticket {
    mailer: Arc<dyn Mailer>,
    redis: Redis,
    to: String,
    /// O dia (UTC) em que o envio contou no teto.
    day: String,
    /// A reserva marcou a espera entre envios para o endereço (os avisos não).
    cooldown: bool,
}

impl Ticket {
    /// Manda e espera o provedor. Falhou → 502 `email.send_failed`, com a
    /// reserva devolvida; quem chamou desfaz o que preparou para o email (o
    /// link que não chegou).
    pub async fn send(self, email: Email) -> Result<(), ApiError> {
        if self.deliver(&email).await {
            Ok(())
        } else {
            Err(ApiError::new(StatusCode::BAD_GATEWAY, "email.send_failed"))
        }
    }

    /// Manda em segundo plano: quem pediu não espera o provedor, e a falha só
    /// vai para o log.
    pub fn send_in_background(self, email: Email) {
        tokio::spawn(async move {
            self.deliver(&email).await;
        });
    }

    /// Manda pelo provedor; falhando, loga e devolve a reserva. Diz se saiu.
    async fn deliver(&self, email: &Email) -> bool {
        debug_assert_eq!(
            email.to, self.to,
            "email para outro endereço que o reservado"
        );
        let Err(err) = self.mailer.send(email).await else {
            return true;
        };
        tracing::error!(error = ?err, subject = %email.subject, "envio de email falhou");
        let to = self.cooldown.then_some(self.to.as_str());
        if let Err(err) = kv::release(&self.redis, to, &self.day).await {
            tracing::error!(error = ?err, "reserva do email que falhou não foi devolvida");
        }
        false
    }
}
