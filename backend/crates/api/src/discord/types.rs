use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::shared::Locale;

/// O webhook como a API devolve ao owner: sem o endereço nem o token.
#[derive(Serialize, ToSchema, Debug, Clone, PartialEq, Eq)]
pub struct WebhookSettings {
    /// O id do webhook (a parte pública do endereço), para reconhecer qual é.
    #[schema(example = "1234567890123456789")]
    pub webhook_id: String,
    /// O Discord disse que o webhook não existe mais: nada é enviado até o
    /// owner colar outro endereço.
    pub broken: bool,
    /// Códigos dos avisos ligados (`check_in.opened`, `match.ready`...).
    pub events: Vec<String>,
    /// Idioma das mensagens.
    pub locale: Locale,
    /// Mencionar (`@`) quem vinculou o Discord à conta nos avisos que se
    /// dirigem a pessoas.
    pub mention_players: bool,
}

/// Um aviso que a comunidade pode ligar.
#[derive(Serialize, ToSchema, Debug, Clone, PartialEq, Eq)]
pub struct AvailableEvent {
    #[schema(example = "match.ready")]
    pub code: String,
    /// Vem ligado numa configuração nova.
    pub default_on: bool,
    /// O aviso se dirige a pessoas (a partida pronta, a organização), então
    /// "mencionar" faz diferença nele.
    pub addressed: bool,
}

#[derive(Serialize, ToSchema, Debug, Clone, PartialEq, Eq)]
pub struct DiscordSettings {
    /// Os avisos que existem, na ordem em que aparecem.
    pub available: Vec<AvailableEvent>,
    /// `null`: a comunidade não tem webhook.
    pub webhook: Option<WebhookSettings>,
}

#[derive(Deserialize, ToSchema)]
pub struct PutDiscord {
    /// `https://discord.com/api/webhooks/{id}/{token}` (também `discordapp.com`,
    /// `ptb.` e `canary.`). Sem ele, mantém o webhook atual e muda só o resto;
    /// com ele, troca o webhook (e tira a marca de quebrado).
    pub url: Option<String>,
    /// Códigos dos avisos ligados.
    pub events: Vec<String>,
    pub locale: Locale,
    pub mention_players: bool,
}
