//! O relógio do veto: aplica os passos vencidos nos torneios que ninguém leu
//! (o `realtime` chama uma passada quando acorda: no próximo prazo ou quando
//! algo muda). Reler a fase de um torneio custa ~9 queries e o replay, então a
//! passada não relê todos: uma query lista os torneios com veto aberto, com o
//! carimbo do histórico do veto de cada um (`db::Stamp`), e só relê o torneio
//! cujo prazo guardado venceu, cujo carimbo mudou, que um evento do outbox
//! alcançou desde a passada anterior (`outbox::Touched`: o elenco e a chave mudam
//! sem evento de veto) ou todos, a cada `REFRESH`. O que a passada viu fica em
//! memória (`VetoClock`) só como atalho: o banco continua mandando, a API que
//! sobe começa relendo tudo, e o torneio que falhou é relido na passada seguinte.

use std::collections::HashMap;

use chrono::{DateTime, TimeDelta, Utc};
use sqlx::PgPool;

use super::{
    db::{self, OpenTournament, Stamp},
    due::{any_due, apply_due, next_deadline},
};
use crate::{
    matches::{changes::Snapshot, settle, stage},
    outbox::Touched,
    shared::ApiError,
    tournaments,
};

/// A cada quanto a passada relê todos os torneios abertos, mesmo sem nada
/// mudar: a rede de segurança do atalho (antes, o agendador relia todos a cada
/// 10 s, e cada requisição que emitia, mais uma vez).
const REFRESH: TimeDelta = TimeDelta::seconds(60);
/// Depois de um torneio que falhou, quando tentar de novo.
const RETRY: TimeDelta = TimeDelta::seconds(10);

/// O que a última leitura de um torneio deu.
struct Known {
    stamp: Stamp,
    /// O próximo prazo (mais `GRACE`); `None`: nenhum passo esperando.
    next: Option<DateTime<Utc>>,
}

impl Known {
    /// Vale sem reler: o histórico do veto é o mesmo e o prazo não venceu.
    fn holds(&self, stamp: Stamp, now: DateTime<Utc>) -> bool {
        self.stamp == stamp && self.next.is_none_or(|next| next > now)
    }
}

/// O que o relógio sabe dos torneios abertos, entre uma passada e outra.
#[derive(Default)]
pub struct VetoClock {
    known: HashMap<i64, Known>,
    /// A última passada que releu todos.
    read_all_at: Option<DateTime<Utc>>,
}

/// O que uma passada fez.
pub struct VetoPass {
    /// Passos sorteados.
    pub written: u64,
    /// Quando passar de novo, se nada acordar o relógio antes: o próximo prazo,
    /// a próxima releitura de todos ou a nova tentativa. Antes de `now` só se o
    /// prazo continua vencido depois de aplicado (não havia o que sortear).
    pub wake_at: DateTime<Utc>,
}

impl VetoClock {
    /// Aplica o que venceu em `now`, cada torneio na própria transação e com
    /// ele travado (só quando há algo vencido: ler não trava). `changed`: o que
    /// os eventos emitidos desde a passada anterior alcançaram. O erro de um
    /// torneio fica no log e não para os outros.
    pub async fn pass(
        &mut self,
        db: &PgPool,
        now: DateTime<Utc>,
        changed: &Touched,
    ) -> Result<VetoPass, ApiError> {
        let open = db::open_tournaments(db).await?;
        let all = changed.reaches_all() || self.read_all_at.is_none_or(|at| now >= at + REFRESH);
        let read_all_at = match self.read_all_at {
            Some(at) if !all => at,
            _ => now,
        };
        self.read_all_at = Some(read_all_at);
        self.known.retain(|id, _| open.iter().any(|t| t.id == *id));
        let mut written = 0;
        let mut wake_at = read_all_at + REFRESH;
        for t in &open {
            let known = self.known.get(&t.id).filter(|k| {
                !all && !changed.reaches(&t.tenant_slug, &t.slug) && k.holds(t.stamp, now)
            });
            if let Some(k) = known {
                wake_at = k.next.map_or(wake_at, |next| wake_at.min(next));
                continue;
            }
            match read(db, t, now).await {
                Ok((n, next)) => {
                    written += n;
                    // O carimbo de antes da leitura: se ela gravou, a próxima
                    // passada relê (uma vez) em vez de confiar no que não viu.
                    let known = Known {
                        stamp: t.stamp,
                        next,
                    };
                    self.known.insert(t.id, known);
                    wake_at = next.map_or(wake_at, |next| wake_at.min(next));
                }
                Err(e) => {
                    tracing::error!(
                        slug = t.slug.as_str(),
                        code = e.code,
                        "tempo do veto falhou"
                    );
                    self.known.remove(&t.id);
                    wake_at = wake_at.min(now + RETRY);
                }
            }
        }
        Ok(VetoPass { written, wake_at })
    }
}

/// Relê a fase de `t` e aplica o que venceu em `now`. Devolve quantos passos
/// gravou e o próximo prazo.
async fn read(
    db: &PgPool,
    t: &OpenTournament,
    now: DateTime<Utc>,
) -> Result<(u64, Option<DateTime<Utc>>), ApiError> {
    let mut conn = db.acquire().await?;
    let Some(found) = tournaments::find(&mut conn, t.tenant, &t.slug, None).await? else {
        return Ok((0, None));
    };
    let current = stage::load(&mut conn, &found).await?;
    if !any_due(found.status, &current, now) {
        return Ok((0, next_deadline(found.status, &current)));
    }
    drop(conn);
    let mut tx = db.begin().await?;
    let Some(found) = tournaments::find_for_update(&mut tx, t.tenant, &t.slug, None).await? else {
        return Ok((0, None));
    };
    let current = stage::load(&mut tx, &found).await?;
    let before = Snapshot::of(&current);
    let n = apply_due(&mut tx, &found, &current, now).await?;
    let (after, _) = settle::settle(&mut tx, &found, None, &before).await?;
    tx.commit().await?;
    Ok((n, next_deadline(found.status, &after)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stamp(events: i64) -> Stamp {
        Stamp {
            events,
            last_id: events,
            last_at: DateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn what_was_read_holds_until_the_history_changes_or_the_deadline_passes() {
        let now = Utc::now();
        let waiting = Known {
            stamp: stamp(3),
            next: Some(now + TimeDelta::seconds(30)),
        };
        assert!(waiting.holds(stamp(3), now));
        assert!(!waiting.holds(stamp(4), now));
        assert!(!waiting.holds(stamp(3), now + TimeDelta::seconds(30)));
        let idle = Known {
            stamp: stamp(3),
            next: None,
        };
        assert!(idle.holds(stamp(3), now + TimeDelta::days(1)));
        assert!(!idle.holds(stamp(2), now));
    }
}
