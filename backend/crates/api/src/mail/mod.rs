//! Email: o endereço na forma que é gravada (`Mail::normalize`: a canônica,
//! sem descartáveis nem apelidos, ou, com `EMAIL_STRICT` desligado, como foi
//! escrito), o layout no visual do site (`compose`), o envio atrás da trait
//! `Mailer` e os limites de envio no Redis (por endereço, por dia e, nos
//! pedidos sem login, por IP). Não tem rota, e do texto só o nome
//! do site e o rodapé: quem manda (`auth`) escreve o email no idioma de quem
//! recebe, reserva o envio e chama `Ticket::send`. Só depende de `shared`.

mod address;
mod disposable;
mod kv;
mod layout;
mod sender;

use std::sync::Arc;

use axum::http::StatusCode;
use chrono::Utc;

use crate::shared::{ApiError, ClientIp, Redis};

pub use layout::{Action, Alert, Content, compose};
pub use sender::{Email, LogMailer, Mailer, ResendMailer};

/// Espera entre dois envios para o mesmo endereço, em cada fila (`kv::Lane`).
const COOLDOWN_SECS: u64 = 60;

/// Teto de envios por dia quando `EMAIL_DAILY_LIMIT` não diz: abaixo dos 100
/// do plano grátis do Resend.
pub const DEFAULT_DAILY_LIMIT: u32 = 90;

/// Pedidos sem login que mandam email (cadastro e o reenvio dele, entrar por
/// email), somados, por IP: na hora cheia e no dia (UTC). Quem se cadastra
/// pede um ou dois; mais que isso de um IP só é abuso, e o teto do dia (90)
/// não acaba na mão de um IP só.
const IP_HOURLY_LIMIT: u32 = 5;
const IP_DAILY_LIMIT: u32 = 20;

/// Pedidos de uma conta logada que mandam email (a troca, a prova do email de
/// agora). A espera é por endereço, então só isto segura quem muda de endereço.
/// Quem troca o email pede um ou dois, um por erro de digitação; 5 na hora dá
/// folga. Os 20 do dia são porque 5 × 24 passaria dos 90 do teto: uma conta só
/// não gasta mais que um quarto dele.
const ACCOUNT_HOURLY_LIMIT: u32 = 5;
const ACCOUNT_DAILY_LIMIT: u32 = 20;

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

    /// A mesma forma, para procurar a conta de um endereço (entrar com senha ou
    /// por email): não recusa domínio (`address::canonical`).
    pub fn canonical(&self, raw: &str) -> Result<String, ApiError> {
        address::canonical(raw, self.strict)
    }

    /// Reserva um envio para `to` nos limites, antes de gerar o link que vai
    /// nele: assim um pedido recusado não troca o link que já foi. Envio
    /// recente para o mesmo endereço → 429 `email.too_soon` com
    /// `{"seconds"}`; teto do dia → 503 `email.unavailable`.
    pub async fn reserve(&self, redis: &Redis, to: &str) -> Result<Ticket, ApiError> {
        let lane = kv::Lane::Requests;
        match kv::reserve(redis, to, lane, COOLDOWN_SECS, self.daily_limit).await? {
            kv::Slot::Granted { day } => Ok(self.ticket(redis, to, day, Some(lane))),
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

    /// Conta um pedido sem login que manda email (cadastro, entrar por email)
    /// para o IP de quem pede, antes de tudo que gasta (o hash da senha, o
    /// envio): passou de `IP_HOURLY_LIMIT` na hora ou de `IP_DAILY_LIMIT` no
    /// dia → 429 `email.ip_limited` com `{"minutes"}` até o período acabar.
    /// O pedido conta mesmo que o envio depois não saia (o mesmo endereço há
    /// menos de 60 s), para quem insiste não passar do limite. Sem IP (só no
    /// app dos testes), não conta.
    pub async fn limit_ip(&self, redis: &Redis, ip: &ClientIp) -> Result<(), ApiError> {
        let Some(bucket) = ip.bucket() else {
            return Ok(());
        };
        let scope = format!("email_ip:{bucket}");
        let slot =
            kv::count_requests(redis, &scope, IP_HOURLY_LIMIT, IP_DAILY_LIMIT, Utc::now()).await?;
        limited(slot, "email.ip_limited").inspect_err(|_| {
            tracing::warn!(ip = %bucket, "limite de emails por IP atingido");
        })
    }

    /// Conta um pedido de uma conta logada que manda email: passou de
    /// `ACCOUNT_HOURLY_LIMIT` na hora ou de `ACCOUNT_DAILY_LIMIT` no dia → 429
    /// `email.account_limited` com `{"minutes"}` até o período acabar.
    pub async fn limit_account(&self, redis: &Redis, user_id: i64) -> Result<(), ApiError> {
        let scope = format!("email_account:{user_id}");
        let slot = kv::count_requests(
            redis,
            &scope,
            ACCOUNT_HOURLY_LIMIT,
            ACCOUNT_DAILY_LIMIT,
            Utc::now(),
        )
        .await?;
        limited(slot, "email.account_limited").inspect_err(|_| {
            tracing::warn!(user = user_id, "limite de emails por conta atingido");
        })
    }

    /// Reserva um envio que nenhuma resposta mostra (entrar por email, o que só
    /// vai à própria conta), na espera à parte de `kv::Lane::Quiet`: fora dos
    /// limites devolve `None`, e o email só não sai. Assim a espera não vira
    /// `email.too_soon` no cadastro, o que diria quem tem conta, e um aviso a
    /// este endereço não segura o email de entrar.
    pub async fn reserve_quietly(
        &self,
        redis: &Redis,
        to: &str,
    ) -> Result<Option<Ticket>, ApiError> {
        let lane = kv::Lane::Quiet;
        Ok(
            match kv::reserve(redis, to, lane, COOLDOWN_SECS, self.daily_limit).await? {
                kv::Slot::Granted { day } => Some(self.ticket(redis, to, day, Some(lane))),
                kv::Slot::TooSoon { .. } => None,
                kv::Slot::DailyLimit => {
                    tracing::warn!("teto diário de emails atingido");
                    None
                }
            },
        )
    }

    /// Reserva um aviso (o que mudou na conta) para `to`: conta no teto do
    /// dia, mas não espera nem marca a espera entre envios, que é dos links.
    /// Assim o aviso logo depois de um link (a senha nova ao entrar por email)
    /// não se perde, e não atrasa o próximo link. Fora do teto,
    /// `None`: o aviso só não sai.
    pub async fn reserve_notice(
        &self,
        redis: &Redis,
        to: &str,
    ) -> Result<Option<Ticket>, ApiError> {
        match kv::reserve_today(redis, self.daily_limit).await? {
            kv::Slot::Granted { day } => Ok(Some(self.ticket(redis, to, day, None))),
            _ => {
                tracing::warn!("teto diário de emails atingido; aviso não enviado");
                Ok(None)
            }
        }
    }

    fn ticket(&self, redis: &Redis, to: &str, day: String, cooldown: Option<kv::Lane>) -> Ticket {
        Ticket {
            mailer: self.mailer.clone(),
            redis: redis.clone(),
            to: to.to_owned(),
            day,
            cooldown,
        }
    }
}

/// O limite de pedidos como resposta: fora dele, 429 `code` com `{"minutes"}`.
fn limited(slot: kv::RequestSlot, code: &'static str) -> Result<(), ApiError> {
    match slot {
        kv::RequestSlot::Granted => Ok(()),
        kv::RequestSlot::Limited { seconds } => {
            let minutes = (seconds + 59) / 60;
            Err(ApiError::new(StatusCode::TOO_MANY_REQUESTS, code)
                .with_params([("minutes", minutes.max(1))]))
        }
    }
}

/// Um envio reservado para um endereço. Quem pediu e espera a resposta (o
/// link do cadastro, da troca de email) manda com `send`, que espera o
/// provedor; o que ninguém espera na tela vai em segundo plano, por
/// `send_in_background` (exclusão agendada) ou por `send` dentro da própria
/// tarefa de quem manda (avisos, entrar por email, que desfaz o link). Nos dois, a falha do provedor
/// vai para o log e devolve a reserva: a espera de 60 s sai e o envio não
/// conta no teto do dia, então dá para pedir de novo na hora.
#[must_use = "a reserva gasta o limite; mande o email com `send` ou `send_in_background`"]
pub struct Ticket {
    mailer: Arc<dyn Mailer>,
    redis: Redis,
    to: String,
    /// O dia (UTC) em que o envio contou no teto.
    day: String,
    /// A fila em que a reserva marcou a espera entre envios (os avisos não marcam).
    cooldown: Option<kv::Lane>,
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
        tracing::error!(error = format_args!("{err:#}"), subject = %email.subject, "envio de email falhou");
        let to = self.cooldown.map(|lane| (lane, self.to.as_str()));
        if let Err(err) = kv::release(&self.redis, to, &self.day).await {
            tracing::error!(error = ?err, "reserva do email que falhou não foi devolvida");
        }
        false
    }
}
