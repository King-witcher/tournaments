//! O que a requisição emitiu (`Touched`): a camada de `lib.rs` acorda o
//! despachante só depois das que emitiram (já com a transação encerrada), e o
//! relógio do veto só relê os torneios que os eventos alcançam. Fora de uma
//! requisição (agendador, relógio, socket), quem emite também passa por `track`
//! ou acorda por conta própria. E de qual aba veio a requisição (`origin`), que
//! vai com cada evento dela.

use std::{cell::RefCell, collections::HashSet, future::Future};

use super::event::Event;

tokio::task_local! {
    static EMITTED: RefCell<Touched>;
    static ORIGIN: Option<String>;
}

/// O que os eventos emitidos alcançam: se houve algum (o despachante tem o que
/// fazer) e quais torneios podem ter mudado (o relógio do veto relê só esses).
/// A mudança de uma comunidade (membros, papéis) alcança todos os torneios
/// dela, porque o elenco dos times muda com ela; endereço novo, todos.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Touched {
    any: bool,
    everything: bool,
    tenants: HashSet<String>,
    /// `(comunidade, torneio)`, os slugs.
    tournaments: HashSet<(String, String)>,
}

impl Touched {
    /// Tudo: para quem emitiu fora de `track` e não diz o quê.
    pub fn everything() -> Self {
        Self {
            any: true,
            everything: true,
            ..Self::default()
        }
    }

    fn add(&mut self, event: &Event) {
        self.any = true;
        match event {
            Event::Tournament {
                tenant, tournament, ..
            } => {
                self.tournaments
                    .insert((tenant.clone(), tournament.clone()));
            }
            Event::Tenant { tenant } | Event::AccessLost { tenant, .. } => {
                self.tenants.insert(tenant.clone());
            }
            Event::Moved { .. } => self.everything = true,
            Event::Me { .. }
            | Event::SessionsEnded { .. }
            | Event::Notice { .. }
            | Event::Unread { .. }
            | Event::Announce { .. } => {}
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.any |= other.any;
        self.everything |= other.everything;
        self.tenants.extend(other.tenants);
        self.tournaments.extend(other.tournaments);
    }

    /// Algum evento foi emitido.
    pub const fn any(&self) -> bool {
        self.any
    }

    /// Pode ter mudado qualquer torneio.
    pub const fn reaches_all(&self) -> bool {
        self.everything
    }

    /// Alcança algum torneio.
    pub fn reaches_some(&self) -> bool {
        self.everything || !self.tenants.is_empty() || !self.tournaments.is_empty()
    }

    /// Alcança o torneio `tournament` da comunidade `tenant` (os slugs).
    pub fn reaches(&self, tenant: &str, tournament: &str) -> bool {
        self.everything
            || self.tenants.contains(tenant)
            || self
                .tournaments
                .iter()
                .any(|(a, b)| a == tenant && b == tournament)
    }
}

/// Roda `fut` e devolve, junto com o resultado, o que os `emit` dela
/// alcançaram. `origin`: o id da conexão do WebSocket da aba que pediu (o
/// despachante não devolve a ela as mudanças de partida que ela mesma causou).
pub async fn track<F: Future>(origin: Option<String>, fut: F) -> (F::Output, Touched) {
    ORIGIN
        .scope(
            origin,
            EMITTED.scope(RefCell::new(Touched::default()), async {
                let out = fut.await;
                (out, EMITTED.with(RefCell::take))
            }),
        )
        .await
}

pub(super) fn mark(event: &Event) {
    let _ = EMITTED.try_with(|touched| touched.borrow_mut().add(event));
}

/// A aba de onde veio a requisição em andamento, se ela disse.
pub fn origin() -> Option<String> {
    ORIGIN.try_with(Clone::clone).ok().flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tournament(tenant: &str, slug: &str) -> Event {
        Event::Tournament {
            tenant: tenant.into(),
            tournament: slug.into(),
            match_key: Some("wb-1-1".into()),
        }
    }

    #[test]
    fn a_change_reaches_its_tournament_and_a_tenant_change_all_of_its_tournaments() {
        let mut touched = Touched::default();
        touched.add(&Event::me([1]));
        assert!(touched.any());
        assert!(!touched.reaches_some());
        touched.add(&tournament("quake-br", "copa"));
        assert!(touched.reaches("quake-br", "copa"));
        assert!(!touched.reaches("quake-br", "liga"));
        assert!(!touched.reaches("outra", "copa"));
        touched.add(&Event::AccessLost {
            user: 1,
            tenant: "outra".into(),
        });
        assert!(touched.reaches("outra", "liga"));
        assert!(!touched.reaches_all());
        touched.merge(Touched::everything());
        assert!(touched.reaches("terceira", "qualquer"));
    }

    #[tokio::test]
    async fn track_sees_only_what_was_emitted_inside_it() {
        let ((), touched) = track(None, async { mark(&tournament("quake-br", "copa")) }).await;
        assert!(touched.reaches("quake-br", "copa"));
        let ((), touched) = track(None, async {}).await;
        assert_eq!(touched, Touched::default());
        // Fora de `track`, não há onde marcar (e não quebra).
        mark(&tournament("quake-br", "copa"));
    }
}
