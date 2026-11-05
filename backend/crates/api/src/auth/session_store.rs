//! As sessões em memória: toda requisição logada lê daqui, sem ida ao Redis. O
//! Redis (`kv::sessions`) só guarda entre os boots: a memória sobe dele, e toda
//! escrita vai primeiro a ele (falhou, nada muda e a rota responde 500) e depois
//! à memória. Vale porque a API roda em uma instância só; com duas, uma não veria
//! o logout da outra. Os motivos estão em `docs/decisions/stack.md`, "Redis".

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};

use chrono::{DateTime, Utc};
use redis::RedisResult;
use tokio::sync::Mutex;

use super::kv::{self, StoredSession};
use crate::shared::{Redis, token};

/// Num deploy, a instância antiga atende até a nova ficar no ar, e a nova já leu
/// o Redis ao subir: quem entrou ou saiu nesse meio tempo só aparece nesta segunda
/// leitura. O tempo é uma folga para a antiga já ter parado, não uma medida.
const RELOAD_AFTER_BOOT: Duration = Duration::from_secs(5 * 60);

/// Clonar é barato; os clones dividem a memória.
#[derive(Clone)]
pub struct SessionStore {
    redis: Redis,
    live: Arc<RwLock<Live>>,
    /// Uma escrita por vez, para a recarga não trazer de volta do Redis uma
    /// sessão encerrada enquanto ela lia. Quem só lê a sessão não espera.
    writing: Arc<Mutex<()>>,
    /// Sessão que a memória não conhece é procurada no Redis. Só em
    /// desenvolvimento, onde scripts criam sessão direto nele; em produção, um
    /// cookie inventado gastaria a cota a cada pedido.
    read_through: bool,
}

impl SessionStore {
    /// Sem nenhuma sessão, sem ler o Redis.
    pub fn new(redis: Redis) -> Self {
        Self {
            redis,
            live: Default::default(),
            writing: Default::default(),
            read_through: false,
        }
    }

    /// Com as sessões que estão no Redis. Também é o que confere a conexão ao
    /// subir.
    pub async fn load(redis: Redis, read_through: bool) -> RedisResult<Self> {
        let store = Self {
            read_through,
            ..Self::new(redis)
        };
        store.reload().await?;
        Ok(store)
    }

    /// A mesma memória, com outro Redis: os testes simulam a queda dele.
    pub fn with_redis(&self, redis: Redis) -> Self {
        Self {
            redis,
            ..self.clone()
        }
    }

    /// Troca a memória pelo que está no Redis.
    pub async fn reload(&self) -> RedisResult<()> {
        let _writing = self.writing.lock().await;
        let sessions = kv::load_sessions(&self.redis).await?;
        tracing::info!(sessions = sessions.len(), "sessões lidas do Redis");
        *self.live.write().expect("trava das sessões") = Live::from(sessions);
        Ok(())
    }

    /// Agenda a segunda leitura do Redis (`RELOAD_AFTER_BOOT`).
    pub fn reload_after_boot(&self) {
        let store = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(RELOAD_AFTER_BOOT).await;
            if let Err(err) = store.reload().await {
                tracing::error!(error = ?err, "segunda leitura das sessões falhou");
            }
        });
    }

    /// Cria a sessão e devolve o token.
    pub async fn create(&self, user_id: i64, user_agent: Option<&str>) -> RedisResult<String> {
        let token = token::random_secret();
        let session = StoredSession {
            hash: token::hex(&token::hash_token(&token)),
            user_id,
            created_at: Utc::now(),
            user_agent: user_agent.map(str::to_owned),
        };
        let _writing = self.writing.lock().await;
        kv::save_session(&self.redis, &session).await?;
        self.write().insert(session, Utc::now());
        Ok(token)
    }

    /// Dono da sessão com este hash, se ela não expirou.
    pub(super) async fn owner(&self, hash: &[u8; 32]) -> RedisResult<Option<i64>> {
        let hash = token::hex(hash);
        let owner = self.read().owner(&hash, Utc::now());
        if owner.is_some() || !self.read_through {
            return Ok(owner);
        }
        let Some(session) = kv::load_session(&self.redis, &hash).await? else {
            return Ok(None);
        };
        let owner = session.user_id;
        let _writing = self.writing.lock().await;
        self.write().insert(session, Utc::now());
        Ok(Some(owner))
    }

    /// A sessão de hash `hash` (em hexadecimal) ainda existe.
    pub(super) fn exists(&self, hash: &str) -> bool {
        self.read().owner(hash, Utc::now()).is_some()
    }

    /// Sessões vivas da conta, da mais nova para a mais antiga.
    pub(super) fn of_user(&self, user_id: i64) -> Vec<StoredSession> {
        self.read().of_user(user_id, Utc::now())
    }

    /// Encerra a sessão com este hash e devolve o dono; sem sessão, nada acontece.
    pub(super) async fn end(&self, hash: &[u8; 32]) -> RedisResult<Option<i64>> {
        let hash = token::hex(hash);
        let _writing = self.writing.lock().await;
        let owner = self.read().owner(&hash, Utc::now());
        if owner.is_some() {
            self.remove(std::slice::from_ref(&hash)).await?;
        }
        Ok(owner)
    }

    /// Encerra a sessão com este hash (em hexadecimal), que acabou de sair de
    /// [`Self::of_user`].
    pub(super) async fn end_listed(&self, hash: &str) -> RedisResult<()> {
        let _writing = self.writing.lock().await;
        self.remove(&[hash.to_owned()]).await
    }

    /// Encerra todas as sessões da conta e devolve os hashes (em hexadecimal).
    pub(super) async fn end_all(&self, user_id: i64) -> RedisResult<Vec<String>> {
        self.end_where(user_id, |_| true).await
    }

    /// Encerra as sessões da conta, menos a de hash `keep`, e devolve os hashes
    /// (em hexadecimal) das que encerrou.
    pub(super) async fn end_others(
        &self,
        user_id: i64,
        keep: &[u8; 32],
    ) -> RedisResult<Vec<String>> {
        let keep = token::hex(keep);
        self.end_where(user_id, |hash| hash != keep).await
    }

    async fn end_where(
        &self,
        user_id: i64,
        ends: impl Fn(&str) -> bool,
    ) -> RedisResult<Vec<String>> {
        let _writing = self.writing.lock().await;
        let hashes: Vec<String> = self
            .of_user(user_id)
            .into_iter()
            .map(|s| s.hash)
            .filter(|hash| ends(hash))
            .collect();
        self.remove(&hashes).await?;
        Ok(hashes)
    }

    /// Quem chama segura `writing`.
    async fn remove(&self, hashes: &[String]) -> RedisResult<()> {
        kv::delete_sessions(&self.redis, hashes).await?;
        let mut live = self.write();
        for hash in hashes {
            live.0.remove(hash);
        }
        Ok(())
    }

    fn read(&self) -> std::sync::RwLockReadGuard<'_, Live> {
        self.live.read().expect("trava das sessões")
    }

    fn write(&self) -> std::sync::RwLockWriteGuard<'_, Live> {
        self.live.write().expect("trava das sessões")
    }
}

/// As sessões por hash (em hexadecimal). Listar por conta percorre todas: é raro,
/// e um índice por conta seria mais um estado para manter igual.
#[derive(Default)]
struct Live(HashMap<String, StoredSession>);

impl From<Vec<StoredSession>> for Live {
    fn from(sessions: Vec<StoredSession>) -> Self {
        Self(sessions.into_iter().map(|s| (s.hash.clone(), s)).collect())
    }
}

impl Live {
    /// Guarda a sessão e esquece as vencidas, que nenhuma leitura devolve mais.
    fn insert(&mut self, session: StoredSession, now: DateTime<Utc>) {
        self.0.retain(|_, s| s.expires_at() > now);
        self.0.insert(session.hash.clone(), session);
    }

    fn owner(&self, hash: &str, now: DateTime<Utc>) -> Option<i64> {
        let session = self.0.get(hash)?;
        (session.expires_at() > now).then_some(session.user_id)
    }

    fn of_user(&self, user_id: i64, now: DateTime<Utc>) -> Vec<StoredSession> {
        let mut sessions: Vec<StoredSession> = self
            .0
            .values()
            .filter(|s| s.user_id == user_id && s.expires_at() > now)
            .cloned()
            .collect();
        sessions.sort_by_key(|s| std::cmp::Reverse(s.created_at));
        sessions
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use super::*;
    use crate::auth::session::SESSION_DAYS;

    fn session(hash: &str, user_id: i64, created_at: DateTime<Utc>) -> StoredSession {
        StoredSession {
            hash: hash.into(),
            user_id,
            created_at,
            user_agent: None,
        }
    }

    #[test]
    fn an_expired_session_has_no_owner_and_is_not_listed() {
        let now = Utc::now();
        let lasts = TimeDelta::days(SESSION_DAYS);
        let live = Live::from(vec![
            session("velha", 1, now - lasts),
            session("nova", 1, now - lasts + TimeDelta::seconds(1)),
        ]);
        assert_eq!(live.owner("velha", now), None);
        assert_eq!(live.owner("nova", now), Some(1));
        let listed = live.of_user(1, now);
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].hash, "nova");
    }

    #[test]
    fn a_new_session_sweeps_the_expired_ones() {
        let now = Utc::now();
        let mut live = Live::from(vec![session(
            "velha",
            1,
            now - TimeDelta::days(SESSION_DAYS),
        )]);
        live.insert(session("nova", 2, now), now);
        assert_eq!(live.0.len(), 1);
        assert!(live.0.contains_key("nova"));
    }

    #[test]
    fn sessions_are_listed_per_account_newest_first() {
        let now = Utc::now();
        let live = Live::from(vec![
            session("antiga", 1, now - TimeDelta::hours(2)),
            session("recente", 1, now - TimeDelta::hours(1)),
            session("de-outro", 2, now),
        ]);
        let hashes: Vec<_> = live.of_user(1, now).into_iter().map(|s| s.hash).collect();
        assert_eq!(hashes, ["recente", "antiga"]);
    }
}
