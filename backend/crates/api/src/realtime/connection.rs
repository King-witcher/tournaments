//! O que um socket assina e o que ele recebe de cada evento: só o que a
//! conexão assinou (a assinatura de um torneio também recebe a comunidade
//! dele) e o que é da pessoa dela, menos as mudanças de partida que a própria
//! aba pediu dizendo que aplica a resposta (`origin`). Sem I/O: o laço do
//! socket (`socket`) confere o acesso antes de assinar e manda o que sai
//! daqui.

use std::collections::HashSet;

use super::{
    hub::Published,
    types::{Changed, ServerMessage, Topic},
};
use crate::outbox::Event;

/// Assinaturas por conexão: uma aba assina o que a página mostra.
pub const MAX_TOPICS: usize = 32;

pub struct Connection {
    user: i64,
    /// Id público da sessão do cookie, para fechar quando ela acaba.
    session: String,
    /// O id que a aba deu à conexão (`/ws?connection=`), se deu.
    id: Option<String>,
    topics: HashSet<Topic>,
}

/// O que fazer com um evento.
#[derive(Debug, PartialEq, Eq)]
pub enum Delivery {
    Send(ServerMessage),
    /// A sessão da conexão acabou.
    Close,
    Skip,
}

impl Connection {
    pub fn new(user: i64, session: String, id: Option<String>) -> Self {
        Self {
            user,
            session,
            id,
            topics: HashSet::new(),
        }
    }

    /// O evento veio de uma ação desta aba, que já aplicou a resposta.
    fn own(&self, published: &Published) -> bool {
        self.id.is_some() && published.origin == self.id
    }

    #[cfg(test)]
    pub fn is_subscribed(&self, topic: &Topic) -> bool {
        self.topics.contains(topic)
    }

    /// `false` se já tem o máximo de assinaturas.
    pub fn subscribe(&mut self, topic: Topic) -> bool {
        if self.topics.len() >= MAX_TOPICS && !self.topics.contains(&topic) {
            return false;
        }
        self.topics.insert(topic);
        true
    }

    pub fn unsubscribe(&mut self, topic: &Topic) {
        self.topics.remove(topic);
    }

    pub fn deliver(&mut self, published: &Published) -> Delivery {
        let message = match &published.event {
            // A mudança de partida que esta aba pediu e cuja resposta ela já
            // aplicou: recarregar só repetiria o que ela tem. As do torneio
            // inteiro, de "Seus torneios" e os avisos chegam.
            Event::Tournament {
                match_key: Some(_), ..
            } if self.own(published) => return Delivery::Skip,
            Event::Tenant { tenant } if self.topics.iter().any(|t| t.tenant == *tenant) => {
                ServerMessage::Changed(Changed::Tenant {
                    tenant: tenant.clone(),
                })
            }
            Event::Tournament {
                tenant,
                tournament,
                match_key,
            } if self.topics.contains(&Topic {
                tenant: tenant.clone(),
                tournament: Some(tournament.clone()),
            }) =>
            {
                ServerMessage::Changed(Changed::Tournament {
                    tenant: tenant.clone(),
                    tournament: tournament.clone(),
                    match_key: match_key.clone(),
                })
            }
            Event::Me { users } if users.contains(&self.user) => {
                ServerMessage::Changed(Changed::Me)
            }
            Event::AccessLost { user, tenant } if *user == self.user => {
                self.topics.retain(|t| t.tenant != *tenant);
                ServerMessage::AccessLost {
                    tenant: tenant.clone(),
                }
            }
            Event::SessionsEnded { user, sessions }
                if *user == self.user && sessions.contains(&self.session) =>
            {
                return Delivery::Close;
            }
            // Destinatário é quem tem a notificação guardada.
            Event::Notice { notice, .. } => match published.notifications.get(&self.user) {
                Some(&id) => ServerMessage::Notify {
                    id,
                    notice: notice.clone(),
                },
                None => return Delivery::Skip,
            },
            Event::Unread { user, count } if *user == self.user => {
                ServerMessage::Unread { count: *count }
            }
            _ => return Delivery::Skip,
        };
        Delivery::Send(message)
    }

    /// Depois de perder eventos (fila lenta): uma mudança de tudo o que
    /// assina, e de "Seus torneios", para a aba recarregar.
    pub fn resync(&self) -> Vec<ServerMessage> {
        let mut tenants: Vec<&String> = self.topics.iter().map(|t| &t.tenant).collect();
        tenants.sort();
        tenants.dedup();
        let mut messages: Vec<ServerMessage> = tenants
            .into_iter()
            .map(|tenant| {
                ServerMessage::Changed(Changed::Tenant {
                    tenant: tenant.clone(),
                })
            })
            .collect();
        for topic in &self.topics {
            if let Some(tournament) = &topic.tournament {
                messages.push(ServerMessage::Changed(Changed::Tournament {
                    tenant: topic.tenant.clone(),
                    tournament: tournament.clone(),
                    match_key: None,
                }));
            }
        }
        messages.push(ServerMessage::Changed(Changed::Me));
        messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::outbox::{Notice, NoticeKind};

    fn topic(tenant: &str, tournament: Option<&str>) -> Topic {
        Topic {
            tenant: tenant.into(),
            tournament: tournament.map(Into::into),
        }
    }

    fn tournament(tenant: &str, slug: &str, key: Option<&str>) -> Event {
        Event::Tournament {
            tenant: tenant.into(),
            tournament: slug.into(),
            match_key: key.map(Into::into),
        }
    }

    fn connection() -> Connection {
        let mut c = Connection::new(7, "abcd".into(), None);
        assert!(c.subscribe(topic("quake-br", Some("copa"))));
        assert!(c.subscribe(topic("quake-ar", None)));
        c
    }

    #[test]
    fn only_what_was_subscribed() {
        let mut c = connection();
        let sent = |d: Delivery| matches!(d, Delivery::Send(_));
        assert_eq!(
            c.deliver(&tournament("quake-br", "copa", Some("wb-1-1")).into()),
            Delivery::Send(ServerMessage::Changed(Changed::Tournament {
                tenant: "quake-br".into(),
                tournament: "copa".into(),
                match_key: Some("wb-1-1".into()),
            }))
        );
        assert!(!sent(
            c.deliver(&tournament("quake-br", "liga", None).into())
        ));
        assert!(!sent(
            c.deliver(&tournament("quake-ar", "copa", None).into())
        ));
        // A comunidade chega para quem assina ela ou um torneio dela.
        for tenant in ["quake-br", "quake-ar"] {
            let event = Event::Tenant {
                tenant: tenant.into(),
            };
            assert!(sent(c.deliver(&event.into())), "{tenant}");
        }
        let other = Event::Tenant {
            tenant: "outra".into(),
        };
        assert!(!sent(c.deliver(&other.into())));
    }

    #[test]
    fn what_is_for_the_person() {
        let mut c = Connection::new(7, "abcd".into(), None);
        assert_eq!(
            c.deliver(&Event::me([3, 7]).into()),
            Delivery::Send(ServerMessage::Changed(Changed::Me))
        );
        assert_eq!(c.deliver(&Event::me([3]).into()), Delivery::Skip);
        let notice = Notice {
            kind: NoticeKind::StartDue {
                tournament: "Copa".into(),
            },
            path: "/quake-br/copa".into(),
        };
        // Cada um recebe o id da própria notificação.
        let published = |users: [i64; 2]| Published {
            event: Event::notice(users, notice.clone()),
            notifications: [(users[0], 41), (users[1], 42)].into(),
            origin: None,
        };
        assert_eq!(
            c.deliver(&published([3, 7])),
            Delivery::Send(ServerMessage::Notify {
                id: 42,
                notice: notice.clone()
            })
        );
        assert_eq!(c.deliver(&published([3, 8])), Delivery::Skip);
        let unread = |user| Event::Unread { user, count: 2 };
        assert_eq!(
            c.deliver(&unread(7).into()),
            Delivery::Send(ServerMessage::Unread { count: 2 })
        );
        assert_eq!(c.deliver(&unread(8).into()), Delivery::Skip);
    }

    #[test]
    fn access_lost_drops_the_tenant_subscriptions() {
        let mut c = connection();
        assert!(c.subscribe(topic("quake-br", None)));
        let lost = Event::AccessLost {
            user: 7,
            tenant: "quake-br".into(),
        };
        assert_eq!(
            c.deliver(&lost.into()),
            Delivery::Send(ServerMessage::AccessLost {
                tenant: "quake-br".into()
            })
        );
        assert!(!c.is_subscribed(&topic("quake-br", Some("copa"))));
        assert_eq!(
            c.deliver(&tournament("quake-br", "copa", None).into()),
            Delivery::Skip
        );
        assert!(c.is_subscribed(&topic("quake-ar", None)));
        // O de outra pessoa não muda nada.
        let mut c = connection();
        let other = Event::AccessLost {
            user: 8,
            tenant: "quake-br".into(),
        };
        assert_eq!(c.deliver(&other.into()), Delivery::Skip);
        assert!(c.is_subscribed(&topic("quake-br", Some("copa"))));
    }

    #[test]
    fn the_tab_that_acted_does_not_get_its_own_match_change_back() {
        let tab = "0123456789abcdef";
        let mut c = Connection::new(7, "abcd".into(), Some(tab.into()));
        assert!(c.subscribe(topic("quake-br", Some("copa"))));
        let from = |origin: Option<&str>, event: Event| Published {
            origin: origin.map(Into::into),
            ..event.into()
        };
        let sent = |d: Delivery| matches!(d, Delivery::Send(_));
        let game = || tournament("quake-br", "copa", Some("wb-1-1"));
        assert_eq!(c.deliver(&from(Some(tab), game())), Delivery::Skip);
        // De outra aba, de ninguém (o relógio) ou do torneio inteiro, chega.
        assert!(sent(c.deliver(&from(Some("fedcba9876543210"), game()))));
        assert!(sent(c.deliver(&from(None, game()))));
        let whole = tournament("quake-br", "copa", None);
        assert!(sent(c.deliver(&from(Some(tab), whole))));
        assert!(sent(c.deliver(&from(Some(tab), Event::me([7])))));
        // A aba sem id recebe tudo.
        let mut plain = connection();
        assert!(sent(plain.deliver(&from(Some(tab), game()))));
    }

    #[test]
    fn ended_sessions_close_only_their_sockets() {
        let mut c = connection();
        let ended = |user, session: &str| Event::SessionsEnded {
            user,
            sessions: vec!["0000".into(), session.into()],
        };
        assert_eq!(c.deliver(&ended(7, "abcd").into()), Delivery::Close);
        assert_eq!(c.deliver(&ended(7, "ffff").into()), Delivery::Skip);
        assert_eq!(c.deliver(&ended(8, "abcd").into()), Delivery::Skip);
    }

    #[test]
    fn subscriptions_are_capped() {
        let mut c = Connection::new(7, "abcd".into(), None);
        for i in 0..MAX_TOPICS {
            assert!(c.subscribe(topic(&format!("t{i}"), None)));
        }
        assert!(!c.subscribe(topic("mais", None)));
        assert!(c.subscribe(topic("t0", None)));
        c.unsubscribe(&topic("t0", None));
        assert!(c.subscribe(topic("mais", None)));
    }

    #[test]
    fn resync_covers_everything_subscribed() {
        let c = connection();
        let messages = c.resync();
        assert_eq!(messages.len(), 4);
        assert!(messages.contains(&ServerMessage::Changed(Changed::Tenant {
            tenant: "quake-br".into()
        })));
        assert!(
            messages.contains(&ServerMessage::Changed(Changed::Tournament {
                tenant: "quake-br".into(),
                tournament: "copa".into(),
                match_key: None,
            }))
        );
        assert_eq!(messages.last(), Some(&ServerMessage::Changed(Changed::Me)));
    }

    #[test]
    fn messages_have_the_documented_shape() {
        let json = |m: &ServerMessage| serde_json::to_value(m).unwrap();
        assert_eq!(
            json(&ServerMessage::Changed(Changed::Me)),
            serde_json::json!({ "type": "changed", "scope": "me" })
        );
        assert_eq!(
            json(&ServerMessage::Changed(Changed::Tournament {
                tenant: "quake-br".into(),
                tournament: "copa".into(),
                match_key: None,
            })),
            serde_json::json!({
                "type": "changed", "scope": "tournament",
                "tenant": "quake-br", "tournament": "copa", "match": null
            })
        );
        let notice = Notice {
            kind: NoticeKind::CheckInOpened {
                tournament: "Copa".into(),
            },
            path: "/quake-br/copa".into(),
        };
        assert_eq!(
            json(&ServerMessage::Notify { id: 5, notice }),
            serde_json::json!({
                "type": "notify", "id": 5, "code": "check_in.opened",
                "params": { "tournament": "Copa" }, "path": "/quake-br/copa"
            })
        );
        let subscribe: super::super::types::ClientMessage =
            serde_json::from_str(r#"{"type":"subscribe","tenant":"quake-br"}"#).unwrap();
        assert_eq!(
            subscribe,
            super::super::types::ClientMessage::Subscribe {
                tenant: "quake-br".into(),
                tournament: None
            }
        );
    }
}
