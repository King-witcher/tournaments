//! Mensagens do WebSocket (`GET /ws`), em JSON. Ficam no OpenAPI como
//! componentes (`ServerMessage`, `ClientMessage`), para o front gerar os tipos.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::outbox::Notice;

/// O que uma aba assina: a comunidade, ou um torneio dela, pelo slug de agora.
/// Quando ele muda, a conexão troca as assinaturas (`Connection::deliver`), para
/// não receber as mudanças de quem pegar o endereço antigo.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Topic {
    pub tenant: String,
    #[serde(default)]
    pub tournament: Option<String>,
}

/// Do navegador para a API. A API confere o acesso como na leitura e
/// responde `subscribed` ou `error`.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Passa a receber as mudanças da comunidade (sem `tournament`) ou do
    /// torneio. A assinatura de um torneio também recebe as da comunidade.
    Subscribe {
        tenant: String,
        #[serde(default)]
        tournament: Option<String>,
    },
    Unsubscribe {
        tenant: String,
        #[serde(default)]
        tournament: Option<String>,
    },
}

/// Da API para o navegador.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Algo que a página mostra mudou: peça de novo à API (o dado não vem
    /// aqui).
    Changed(Changed),
    /// Aviso para quem está conectado (o mesmo do Web Push). Também fica no
    /// sininho, com este `id`: soma uma às não lidas, até a pessoa abrir a
    /// página dele (`POST /me/notifications/read`).
    Notify {
        id: i64,
        #[serde(flatten)]
        notice: Notice,
    },
    /// Quantas notificações faltam ler: a primeira mensagem de toda conexão,
    /// e de novo quando a pessoa as lê noutra aba ou aparelho.
    Unread { count: i64 },
    /// Quem está conectado saiu ou foi removido da comunidade: as assinaturas
    /// dela acabaram.
    AccessLost { tenant: String },
    /// O endereço mudou: o da comunidade `tenant` para `to` ou, com
    /// `tournament`, o desse torneio dela para `to`. A assinatura já foi
    /// trocada; a aba que está lá vai para o endereço novo.
    Moved {
        tenant: String,
        tournament: Option<String>,
        to: String,
    },
    /// Assinatura aceita.
    Subscribed {
        tenant: String,
        tournament: Option<String>,
    },
    /// Assinatura desfeita (também se não havia).
    Unsubscribed {
        tenant: String,
        tournament: Option<String>,
    },
    /// Mensagem recusada: `code` é o do erro da API (`tenant.not_found`,
    /// `tournament.not_found`, `realtime.too_many_subscriptions`,
    /// `request.invalid_body`), com a assinatura pedida, se houver.
    Error {
        code: String,
        tenant: Option<String>,
        tournament: Option<String>,
    },
}

/// Da API para o navegador, no socket de uma sala de veto.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RoomMessage {
    /// A sala mudou: peça de novo à API (o dado não vem aqui).
    Changed,
}

/// O que mudou.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "scope", rename_all = "snake_case")]
pub enum Changed {
    /// A comunidade: membros, papéis, apelidos, link de convite, a lista de
    /// torneios.
    Tenant { tenant: String },
    /// O torneio; `match` quando foi numa partida (a página da partida só
    /// recarrega pela própria; a chave, por qualquer uma).
    Tournament {
        tenant: String,
        tournament: String,
        #[serde(rename = "match")]
        match_key: Option<String>,
    },
    /// "Seus torneios" de quem está conectado.
    Me,
}
