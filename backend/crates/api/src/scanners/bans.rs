//! Os bans em memória: toda requisição lê daqui, sem ida ao Redis (`kv.rs` só
//! guarda entre os boots). Vale porque a API roda em uma instância só.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockWriteGuard},
    time::Duration,
};

use axum::http::HeaderMap;
use chrono::Utc;
use redis::RedisResult;

use super::kv;
use crate::shared::{ClientIp, Redis, token};

/// Quanto dura um ban. Mais não compraria nada: quem volta depois disso se bane
/// de novo no primeiro pedido, e o IP pode já ser de outra pessoa.
const BAN_SECONDS: i64 = 7 * 24 * 60 * 60;

/// Bans guardados. Numa enchente de IPs, os novos ficam de fora, só com o 404 do
/// caminho: a memória é o que não pode acabar.
const MAX_BANS: usize = 50_000;

/// A cada quanto os novos vão ao Redis, todos num comando só: a enchente de IPs
/// não gasta a cota, e no máximo este tempo de bans se perde se a API cair.
const SAVE_EVERY: Duration = Duration::from_secs(5 * 60);

/// Como o servidor do Next se identifica. Ele sai sempre pelos IPs da Vercel, que
/// são divididos com outros clientes dela: sem isto, o scanner de um deles
/// derrubaria as páginas do site inteiro.
pub const SECRET_HEADER: &str = "x-internal-secret";

/// Clonar é barato; os clones dividem a memória.
#[derive(Clone)]
pub struct Bans {
    redis: Redis,
    /// Hash do `INTERNAL_API_SECRET`. `None`: tudo desligado.
    secret: Option<Arc<[u8; 32]>>,
    live: Arc<RwLock<Live>>,
}

impl Bans {
    /// Sem nenhum ban, sem ler o Redis.
    pub fn new(redis: Redis, secret: Option<&str>) -> Self {
        Self {
            redis,
            secret: secret.map(|secret| Arc::new(token::hash_token(secret))),
            live: Default::default(),
        }
    }

    /// Com os bans que estão no Redis. Sem conseguir ler, começa sem nenhum: quem
    /// voltar se bane de novo, e isso não vale um deploy que não sobe.
    pub async fn load(redis: Redis, secret: Option<&str>) -> Self {
        let bans = Self::new(redis, secret);
        if bans.secret.is_some() {
            match kv::load(&bans.redis, now()).await {
                Ok(saved) => {
                    tracing::info!(bans = saved.len(), "bans lidos do Redis");
                    bans.write().until = saved.into_iter().collect();
                }
                Err(err) => tracing::error!(error = ?err, "bans não foram lidos do Redis"),
            }
        }
        bans
    }

    /// Guarda os novos a cada `SAVE_EVERY`.
    pub fn spawn_saver(&self) {
        let bans = self.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(SAVE_EVERY);
            loop {
                tick.tick().await;
                bans.save().await;
            }
        });
    }

    /// Manda os novos ao Redis. Falhou, eles ficam para a próxima.
    pub async fn save(&self) {
        let unsaved = std::mem::take(&mut self.write().unsaved);
        if unsaved.is_empty() {
            return;
        }
        if let Err(err) = kv::save(&self.redis, &unsaved, BAN_SECONDS).await {
            tracing::error!(error = ?err, "bans não foram para o Redis");
            self.write().unsaved.extend(unsaved);
        }
    }

    /// O pedido é do servidor do Next: nem bane nem é barrado.
    pub fn trusted(&self, headers: &HeaderMap) -> bool {
        let Some(secret) = &self.secret else {
            return false;
        };
        headers.get(SECRET_HEADER).is_some_and(|value| {
            let sent = token::hash_token(value.to_str().unwrap_or_default());
            token::constant_time_eq(&sent, secret.as_slice())
        })
    }

    pub fn is_banned(&self, ip: &ClientIp) -> bool {
        ip.bucket().is_some_and(|bucket| {
            let live = self.live.read().expect("trava dos bans");
            live.banned(&bucket, now())
        })
    }

    /// Bane o IP. `why` vai para o log, para achar o motivo se alguém reclamar.
    /// Endereço que não é público fica de fora: sem o `CF-Connecting-IP`, é o do
    /// proxy do Render, por onde passa todo mundo.
    pub fn ban(&self, ip: &ClientIp, why: &str) {
        let (Some(_), Some(bucket)) = (&self.secret, ip.public_bucket()) else {
            return;
        };
        let banned = self.write().ban(bucket.clone(), now());
        if banned {
            let why: String = why.chars().take(100).collect();
            tracing::info!(ip = %bucket, why = ?why, "scanner banido");
        }
    }

    /// Tira o ban na hora, em caso de engano.
    pub async fn unban(&self, ip: &ClientIp) -> RedisResult<()> {
        let Some(bucket) = ip.bucket() else {
            return Ok(());
        };
        kv::delete(&self.redis, &bucket).await?;
        let mut live = self.write();
        live.until.remove(&bucket);
        live.unsaved.retain(|(_, saved)| *saved != bucket);
        Ok(())
    }

    /// Os IPs banidos agora (os de IPv6, por bloco, como em `ClientIp::bucket`).
    pub fn list(&self) -> Vec<String> {
        let now = now();
        let live = self.live.read().expect("trava dos bans");
        live.until
            .iter()
            .filter(|(_, until)| **until > now)
            .map(|(bucket, _)| bucket.clone())
            .collect()
    }

    fn write(&self) -> RwLockWriteGuard<'_, Live> {
        self.live.write().expect("trava dos bans")
    }
}

fn now() -> i64 {
    Utc::now().timestamp()
}

#[derive(Default)]
struct Live {
    /// IP (ou bloco) → fim do ban, em segundos Unix.
    until: HashMap<String, i64>,
    /// Os que ainda não foram ao Redis: (fim do ban, IP).
    unsaved: Vec<(i64, String)>,
    /// A última varredura dos vencidos, com o mapa cheio.
    swept_at: i64,
}

impl Live {
    fn banned(&self, bucket: &str, now: i64) -> bool {
        self.until.get(bucket).is_some_and(|until| *until > now)
    }

    /// Falso se já estava banido ou se não coube.
    fn ban(&mut self, bucket: String, now: i64) -> bool {
        if self.banned(&bucket, now) {
            return false;
        }
        if self.until.len() >= MAX_BANS {
            // Uma vez por minuto: varrer o mapa segura a trava que toda
            // requisição lê, e a enchente pediria isto a cada IP novo.
            if now - self.swept_at >= 60 {
                self.until.retain(|_, until| *until > now);
                self.swept_at = now;
            }
            if self.until.len() >= MAX_BANS {
                return false;
            }
        }
        let until = now + BAN_SECONDS;
        self.until.insert(bucket.clone(), until);
        self.unsaved.push((until, bucket));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ban_ends() {
        let mut live = Live::default();
        assert!(live.ban("a".into(), 100));
        assert!(live.banned("a", 100 + BAN_SECONDS - 1));
        assert!(!live.banned("a", 100 + BAN_SECONDS));
        assert!(!live.banned("b", 100));
    }

    #[test]
    fn banning_twice_saves_once() {
        let mut live = Live::default();
        assert!(live.ban("a".into(), 100));
        assert!(!live.ban("a".into(), 200));
        assert_eq!(live.unsaved.len(), 1);
    }

    #[test]
    fn a_flood_of_ips_does_not_grow_the_map() {
        let mut live = Live::default();
        for n in 0..MAX_BANS {
            assert!(live.ban(n.to_string(), 100));
        }
        assert!(!live.ban("novo".into(), 100));
        assert_eq!(live.until.len(), MAX_BANS);
        // Vencidos dão lugar.
        assert!(live.ban("novo".into(), 100 + BAN_SECONDS));
        assert_eq!(live.until.len(), 1);
    }
}
