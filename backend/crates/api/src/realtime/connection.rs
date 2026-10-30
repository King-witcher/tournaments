//! O que um socket assina e o que ele recebe de cada evento: só o que a
//! conexão assinou (a assinatura de um torneio também recebe a comunidade
//! dele) e o que é da pessoa dela. Sem I/O: o laço do socket (`socket`)
//! confere o acesso antes de assinar e manda o que sai daqui.

use std::collections::HashSet;

use super::types::{Changed, ServerMessage, Topic};
use crate::outbox::Event;

/// Assinaturas por conexão: uma aba assina o que a página mostra.
pub const MAX_TOPICS: usize = 32;

pub struct Connection {
    user: i64,
    /// Id público da sessão do cookie, para fechar quando ela acaba.
    session: String,
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
    pub fn new(user: i64, session: String) -> Self {
        Self {
            user,
            session,
            topics: HashSet::new(),
        }
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

    pub fn deliver(&mut self, event: &Event) -> Delivery {
        let message = match event {
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
            Event::Notice { users, notice } if users.contains(&self.user) => {
                ServerMessage::Notify(notice.clone())
            }
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
        let mut c = Connection::new(7, "abcd".into());
        assert!(c.subscribe(topic("quake-br", Some("copa"))));
        assert!(c.subscribe(topic("quake-ar", None)));
        c
    }

    #[test]
    fn only_what_was_subscribed() {
        let mut c = connection();
        let sent = |d: Delivery| matches!(d, Delivery::Send(_));
        assert_eq!(
            c.deliver(&tournament("quake-br", "copa", Some("wb-1-1"))),
            Delivery::Send(ServerMessage::Changed(Changed::Tournament {
                tenant: "quake-br".into(),
                tournament: "copa".into(),
                match_key: Some("wb-1-1".into()),
            }))
        );
        assert!(!sent(c.deliver(&tournament("quake-br", "liga", None))));
        assert!(!sent(c.deliver(&tournament("quake-ar", "copa", None))));
        // A comunidade chega para quem assina ela ou um torneio dela.
        for tenant in ["quake-br", "quake-ar"] {
            let event = Event::Tenant {
                tenant: tenant.into(),
            };
            assert!(sent(c.deliver(&event)), "{tenant}");
        }
        let other = Event::Tenant {
            tenant: "outra".into(),
        };
        assert!(!sent(c.deliver(&other)));
    }

    #[test]
    fn what_is_for_the_person() {
        let mut c = Connection::new(7, "abcd".into());
        assert_eq!(
            c.deliver(&Event::me([3, 7])),
            Delivery::Send(ServerMessage::Changed(Changed::Me))
        );
        assert_eq!(c.deliver(&Event::me([3])), Delivery::Skip);
        let notice = Notice {
            kind: NoticeKind::StartDue {
                tournament: "Copa".into(),
            },
            path: "/quake-br/copa".into(),
        };
        assert_eq!(
            c.deliver(&Event::notice([7], notice.clone())),
            Delivery::Send(ServerMessage::Notify(notice.clone()))
        );
        assert_eq!(c.deliver(&Event::notice([8], notice)), Delivery::Skip);
        let unread = |user| Event::Unread { user, count: 2 };
        assert_eq!(
            c.deliver(&unread(7)),
            Delivery::Send(ServerMessage::Unread { count: 2 })
        );
        assert_eq!(c.deliver(&unread(8)), Delivery::Skip);
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
            c.deliver(&lost),
            Delivery::Send(ServerMessage::AccessLost {
                tenant: "quake-br".into()
            })
        );
        assert!(!c.is_subscribed(&topic("quake-br", Some("copa"))));
        assert_eq!(
            c.deliver(&tournament("quake-br", "copa", None)),
            Delivery::Skip
        );
        assert!(c.is_subscribed(&topic("quake-ar", None)));
        // O de outra pessoa não muda nada.
        let mut c = connection();
        let other = Event::AccessLost {
            user: 8,
            tenant: "quake-br".into(),
        };
        assert_eq!(c.deliver(&other), Delivery::Skip);
        assert!(c.is_subscribed(&topic("quake-br", Some("copa"))));
    }

    #[test]
    fn ended_sessions_close_only_their_sockets() {
        let mut c = connection();
        let ended = |user, session: &str| Event::SessionsEnded {
            user,
            sessions: vec!["0000".into(), session.into()],
        };
        assert_eq!(c.deliver(&ended(7, "abcd")), Delivery::Close);
        assert_eq!(c.deliver(&ended(7, "ffff")), Delivery::Skip);
        assert_eq!(c.deliver(&ended(8, "abcd")), Delivery::Skip);
    }

    #[test]
    fn subscriptions_are_capped() {
        let mut c = Connection::new(7, "abcd".into());
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
            json(&ServerMessage::Notify(notice)),
            serde_json::json!({
                "type": "notify", "code": "check_in.opened",
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
