//! Limite de pedidos por chave numa janela de tempo, em memória: serve à rota
//! que um abuso chamaria sem parar, onde contar no Redis gastaria a cota dele.
//! Um deploy zera as contagens, e a API roda em uma instância só.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Chaves guardadas antes de limpar as vencidas. Numa enchente de chaves
/// diferentes, tudo é esquecido: a memória é o que não pode acabar, e o limite
/// não é a única defesa de quem o usa.
const MAX_KEYS: usize = 10_000;

/// Clonar é barato; os clones dividem as contagens.
#[derive(Clone, Default)]
pub struct WindowLimit {
    windows: Arc<Mutex<HashMap<String, Window>>>,
}

struct Window {
    count: u32,
    ends: Instant,
}

impl WindowLimit {
    /// Conta um pedido de `key`. Passou de `limit` na janela de `window`, que
    /// começa no primeiro pedido: diz quanto falta para ela acabar.
    pub fn hit(&self, key: String, limit: u32, window: Duration) -> Result<(), Duration> {
        self.hit_at(key, limit, window, Instant::now())
    }

    /// Zera a contagem de `key`.
    pub fn forget(&self, key: &str) {
        self.windows.lock().expect("trava dos limites").remove(key);
    }

    pub(crate) fn hit_at(
        &self,
        key: String,
        limit: u32,
        window: Duration,
        now: Instant,
    ) -> Result<(), Duration> {
        let mut windows = self.windows.lock().expect("trava dos limites");
        if windows.len() >= MAX_KEYS {
            windows.retain(|_, w| w.ends > now);
            if windows.len() >= MAX_KEYS {
                windows.clear();
            }
        }
        let fresh = || Window {
            count: 0,
            ends: now + window,
        };
        let current = windows.entry(key).or_insert_with(fresh);
        if current.ends <= now {
            *current = fresh();
        }
        current.count = current.count.saturating_add(1);
        if current.count > limit {
            return Err(current.ends - now);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINUTE: Duration = Duration::from_secs(60);

    #[test]
    fn counts_per_key_until_the_window_ends() {
        let limit = WindowLimit::default();
        let now = Instant::now();
        let hit = |key: &str, at| limit.hit_at(key.into(), 2, MINUTE, at);
        assert!(hit("a", now).is_ok());
        assert!(hit("a", now).is_ok());
        let later = now + Duration::from_secs(10);
        assert_eq!(hit("a", later), Err(Duration::from_secs(50)));
        assert!(hit("b", now).is_ok());
        assert!(hit("a", now + MINUTE).is_ok());
    }

    #[test]
    fn forgetting_a_key_starts_it_over() {
        let limit = WindowLimit::default();
        let hit = |key: &str| limit.hit(key.into(), 1, MINUTE);
        assert!(hit("a").is_ok());
        assert!(hit("b").is_ok());
        assert!(hit("a").is_err());
        limit.forget("a");
        assert!(hit("a").is_ok());
        assert!(hit("b").is_err());
    }

    #[test]
    fn a_flood_of_keys_does_not_grow_the_map() {
        let limit = WindowLimit::default();
        let now = Instant::now();
        for n in 0..=MAX_KEYS {
            limit.hit_at(n.to_string(), 1, MINUTE, now).unwrap();
        }
        assert_eq!(limit.windows.lock().unwrap().len(), 1);
    }
}
