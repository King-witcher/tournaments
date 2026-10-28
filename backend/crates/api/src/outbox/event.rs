//! O que vai para o outbox: o aviso de mudança (quem precisa recarregar o
//! quê, nunca o dado) e os avisos para pessoas, com código, parâmetros e o
//! caminho, nunca texto. Gravado como jsonb e lido pelo despachante, então
//! mudar o serde daqui pede cuidado com as linhas que ainda não saíram (as
//! que ele não entende, ele descarta com log).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    /// A comunidade mudou: membros, papéis, apelidos, convites ou a lista de
    /// torneios. Só o slug, que não muda.
    Tenant { tenant: String },
    /// O torneio mudou; `match` quando foi numa partida.
    Tournament {
        tenant: String,
        tournament: String,
        #[serde(rename = "match")]
        match_key: Option<String>,
    },
    /// "Seus torneios" destas pessoas mudou.
    Me { users: Vec<i64> },
    /// `user` saiu ou foi removido da comunidade: perde as assinaturas dela.
    AccessLost { user: i64, tenant: String },
    /// Sessões de `user` encerradas (o id público, 16 caracteres do hash): os
    /// sockets delas fecham.
    SessionsEnded { user: i64, sessions: Vec<String> },
    /// Aviso para pessoas, pelo WebSocket e pelo Web Push.
    Notice { users: Vec<i64>, notice: Notice },
}

impl Event {
    /// "Seus torneios" de `users` mudou (sem repetir, em ordem: o mesmo
    /// conjunto dá o mesmo evento, que entra uma vez por transação).
    pub fn me(users: impl IntoIterator<Item = i64>) -> Self {
        Self::Me {
            users: sorted(users),
        }
    }

    pub fn notice(users: impl IntoIterator<Item = i64>, notice: Notice) -> Self {
        Self::Notice {
            users: sorted(users),
            notice,
        }
    }

    /// Evento sem destinatário, que não precisa sair.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Me { users } | Self::Notice { users, .. } => users.is_empty(),
            Self::SessionsEnded { sessions, .. } => sessions.is_empty(),
            _ => false,
        }
    }
}

fn sorted(users: impl IntoIterator<Item = i64>) -> Vec<i64> {
    let mut users: Vec<i64> = users.into_iter().collect();
    users.sort_unstable();
    users.dedup();
    users
}

/// Aviso para uma pessoa: o front traduz `code` com `params` e leva a `path`.
/// É também o corpo do Web Push, com o `locale` da inscrição ao lado.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    #[serde(flatten)]
    pub kind: NoticeKind,
    /// Caminho no site, sem o idioma (ex.: `/quake-br/copa/matches/wb-1-1`).
    #[schema(example = "/quake-br/copa/matches/wb-1-1")]
    pub path: String,
}

/// Código do aviso e os parâmetros dele. `tournament` é o nome do torneio no
/// idioma original; `match`, a partida (`wb-1-3`); `opponent` e `player`,
/// apelidos na comunidade.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "code", content = "params")]
pub enum NoticeKind {
    /// O check-in abriu (para os inscritos).
    #[serde(rename = "check_in.opened")]
    CheckInOpened { tournament: String },
    /// A partida tem os dois lados e pode ser jogada (para os dois).
    #[serde(rename = "match.ready")]
    MatchReady {
        tournament: String,
        #[serde(rename = "match")]
        match_key: String,
        opponent: String,
    },
    /// O adversário enviou um mapa; sem resposta até `confirm_by`, ele
    /// confirma sozinho.
    #[serde(rename = "match.game_reported")]
    GameReported {
        tournament: String,
        #[serde(rename = "match")]
        match_key: String,
        game: i16,
        opponent: String,
        confirm_by: DateTime<Utc>,
    },
    /// Um mapa foi contestado e espera a organização (para organizers e
    /// owner).
    #[serde(rename = "match.game_contested")]
    GameContested {
        tournament: String,
        #[serde(rename = "match")]
        match_key: String,
        game: i16,
        /// Quem contestou.
        player: String,
    },
    /// O início marcado chegou e o torneio não foi iniciado (para organizers e
    /// owner, uma vez).
    #[serde(rename = "tournament.start_due")]
    StartDue { tournament: String },
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn the_stored_shape_is_fixed() {
        let ready = Event::notice(
            [3, 1, 3],
            Notice {
                kind: NoticeKind::MatchReady {
                    tournament: "Copa".into(),
                    match_key: "wb-1-1".into(),
                    opponent: "Bia".into(),
                },
                path: "/quake-br/copa/matches/wb-1-1".into(),
            },
        );
        assert_eq!(
            serde_json::to_value(&ready).unwrap(),
            json!({
                "kind": "notice",
                "users": [1, 3],
                "notice": {
                    "code": "match.ready",
                    "params": { "tournament": "Copa", "match": "wb-1-1", "opponent": "Bia" },
                    "path": "/quake-br/copa/matches/wb-1-1"
                }
            })
        );
        let changed = Event::Tournament {
            tenant: "quake-br".into(),
            tournament: "copa".into(),
            match_key: None,
        };
        let value = json!({ "kind": "tournament", "tenant": "quake-br", "tournament": "copa", "match": null });
        assert_eq!(serde_json::to_value(&changed).unwrap(), value);
        assert_eq!(serde_json::from_value::<Event>(value).unwrap(), changed);
    }

    #[test]
    fn events_without_anyone_are_empty() {
        assert!(Event::me([]).is_empty());
        assert!(!Event::me([1]).is_empty());
        assert!(!Event::Tenant { tenant: "x".into() }.is_empty());
    }
}
