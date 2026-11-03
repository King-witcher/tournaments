//! A saída para um serviço externo que bloqueia quem insiste: um teto de
//! ritmo, para o site nunca ser quem causa o bloqueio, e uma pausa, para não
//! bater no serviço enquanto ele manda esperar. Fica em memória: a API roda em
//! uma instância só, e um Redis no caminho seria consumido pelo próprio abuso.

use std::{
    error::Error,
    fmt,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Por que a chamada não pode sair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    /// O serviço mandou esperar, e a pausa ainda não acabou.
    Paused { retry_after: Duration },
    /// Há mais chamadas na frente do que a espera máxima comporta.
    Busy { retry_after: Duration },
}

impl Refused {
    pub fn retry_after(self) -> Duration {
        match self {
            Self::Paused { retry_after } | Self::Busy { retry_after } => retry_after,
        }
    }
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Paused { retry_after } => write!(f, "saída em pausa por {retry_after:?}"),
            Self::Busy { retry_after } => write!(f, "saída ocupada por {retry_after:?}"),
        }
    }
}

impl Error for Refused {}

/// Clonar é barato; os clones dividem o teto e a pausa.
#[derive(Clone)]
pub struct OutboundGate {
    interval: Duration,
    /// Quanto o horário da próxima chamada pode estar à frente sem esperar: a
    /// rajada que sai na hora.
    burst: Duration,
    max_wait: Duration,
    state: Arc<Mutex<State>>,
}

struct State {
    /// Quando a próxima chamada sairia no ritmo, sem contar a rajada.
    next: Instant,
    paused_until: Instant,
}

impl OutboundGate {
    /// Uma chamada a cada `interval`, com `burst` de uma vez; a chamada que
    /// esperaria mais que `max_wait` é recusada.
    pub fn new(interval: Duration, burst: u32, max_wait: Duration) -> Self {
        let now = Instant::now();
        Self {
            interval,
            burst: interval * burst.saturating_sub(1),
            max_wait,
            state: Arc::new(Mutex::new(State {
                next: now,
                paused_until: now,
            })),
        }
    }

    /// Espera a vez da chamada, ou diz por que ela não sai. A pausa é conferida
    /// de novo depois da espera, porque outra chamada pode ter sido bloqueada.
    pub async fn admit(&self) -> Result<(), Refused> {
        let wait = self.reserve(Instant::now())?;
        if !wait.is_zero() {
            tokio::time::sleep(wait).await;
            self.check_pause(Instant::now())?;
        }
        Ok(())
    }

    /// Nenhuma chamada sai por `duration`. Nunca encurta a pausa em andamento.
    pub fn pause(&self, duration: Duration) {
        self.pause_at(Instant::now(), duration);
    }

    fn pause_at(&self, now: Instant, duration: Duration) {
        let mut state = self.state.lock().expect("trava da saída");
        state.paused_until = state.paused_until.max(now + duration);
    }

    fn check_pause(&self, now: Instant) -> Result<(), Refused> {
        let state = self.state.lock().expect("trava da saída");
        paused(&state, now)
    }

    /// Reserva a vez e diz quanto esperar por ela.
    fn reserve(&self, now: Instant) -> Result<Duration, Refused> {
        let mut state = self.state.lock().expect("trava da saída");
        paused(&state, now)?;
        let next = state.next.max(now);
        let wait = next.saturating_duration_since(now + self.burst);
        if wait > self.max_wait {
            return Err(Refused::Busy {
                retry_after: self.max_wait,
            });
        }
        state.next = next + self.interval;
        Ok(wait)
    }
}

fn paused(state: &State, now: Instant) -> Result<(), Refused> {
    let retry_after = state.paused_until.saturating_duration_since(now);
    if retry_after.is_zero() {
        Ok(())
    } else {
        Err(Refused::Paused { retry_after })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECOND: Duration = Duration::from_secs(1);

    fn gate() -> OutboundGate {
        OutboundGate::new(SECOND, 2, 2 * SECOND)
    }

    #[test]
    fn the_burst_goes_at_once_then_calls_wait_until_the_limit() {
        let gate = gate();
        let now = Instant::now();
        let waits: Vec<_> = (0..4).map(|_| gate.reserve(now).unwrap()).collect();
        assert_eq!(waits, [Duration::ZERO, Duration::ZERO, SECOND, 2 * SECOND]);
        assert_eq!(
            gate.reserve(now),
            Err(Refused::Busy {
                retry_after: 2 * SECOND
            })
        );
    }

    #[test]
    fn the_rate_comes_back_with_time_and_a_refused_call_takes_no_turn() {
        let gate = gate();
        let now = Instant::now();
        for _ in 0..4 {
            gate.reserve(now).unwrap();
        }
        assert!(gate.reserve(now).is_err());
        assert_eq!(gate.reserve(now + SECOND), Ok(2 * SECOND));
        assert_eq!(gate.reserve(now + 10 * SECOND), Ok(Duration::ZERO));
    }

    #[test]
    fn a_pause_refuses_everything_until_it_ends_and_never_shrinks() {
        let gate = gate();
        let now = Instant::now();
        gate.pause_at(now, 5 * SECOND);
        gate.pause_at(now, SECOND);
        assert_eq!(
            gate.reserve(now + SECOND),
            Err(Refused::Paused {
                retry_after: 4 * SECOND
            })
        );
        assert_eq!(gate.reserve(now + 5 * SECOND), Ok(Duration::ZERO));
    }

    #[tokio::test]
    async fn a_pause_during_the_wait_refuses_the_call() {
        let gate = OutboundGate::new(Duration::from_millis(20), 1, SECOND);
        gate.admit().await.unwrap();
        let waiting = {
            let gate = gate.clone();
            tokio::spawn(async move { gate.admit().await })
        };
        tokio::task::yield_now().await;
        gate.pause(SECOND);
        assert!(matches!(
            waiting.await.unwrap(),
            Err(Refused::Paused { .. })
        ));
    }
}
