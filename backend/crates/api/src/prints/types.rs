use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::object::ImageType;

/// O que o front precisa saber dos prints antes de mostrar qualquer coisa.
#[derive(Serialize, ToSchema)]
pub struct PrintSettings {
    /// O bucket está configurado. Sem ele, os prints somem da interface e o
    /// print obrigatório não liga.
    pub enabled: bool,
    /// Tamanho máximo de um print, em bytes.
    #[schema(example = 2097152)]
    pub max_bytes: u32,
    /// Prints por mapa.
    #[schema(example = 3)]
    pub max_per_game: u8,
    pub content_types: Vec<ImageType>,
}

/// Os prints de uma partida, mapa a mapa.
#[derive(Serialize, ToSchema)]
pub struct MatchPrints {
    /// O bucket está configurado (sem ele, `games` vem vazio).
    pub enabled: bool,
    /// O envio do jogador exige um print anexado ao mapa.
    pub required: bool,
    pub max_bytes: u32,
    pub max_per_game: u8,
    /// Os mapas enviados e, por último, o próximo (enquanto a série aceita).
    pub games: Vec<GamePrints>,
}

#[derive(Serialize, ToSchema)]
pub struct GamePrints {
    pub number: i16,
    /// Quem vê pode anexar mais um print a este mapa agora.
    pub can_attach: bool,
    /// Do mais antigo ao mais novo.
    pub prints: Vec<Print>,
}

#[derive(Serialize, ToSchema)]
pub struct Print {
    pub id: i64,
    /// Endereço assinado da imagem, que expira (em pelo menos uma hora); a
    /// mesma na mesma hora, para o navegador usar o cache.
    pub url: String,
    pub content_type: ImageType,
    /// Quem anexou; `null` se a conta não existe mais.
    pub uploaded_by: Option<PrintAuthor>,
    pub attached_at: DateTime<Utc>,
    /// Quem vê pode remover este print.
    pub can_remove: bool,
}

#[derive(Serialize, ToSchema)]
pub struct PrintAuthor {
    pub user_id: i64,
    /// Apelido na comunidade.
    pub nickname: String,
}

/// O arquivo que o navegador vai mandar.
#[derive(Deserialize, ToSchema)]
pub struct NewUpload {
    pub content_type: ImageType,
    /// Tamanho exato, em bytes (o bucket recusa outro).
    #[schema(example = 245_000)]
    pub size: u32,
}

/// Para onde mandar o arquivo: `PUT url` com o `Content-Type` dado, antes de
/// `expires_at`. Depois, `POST .../prints/{id}/attach`.
#[derive(Serialize, ToSchema)]
pub struct PrintUpload {
    pub id: i64,
    pub url: String,
    pub content_type: ImageType,
    pub expires_at: DateTime<Utc>,
}

/// `{tournament}` e `{match}` de `.../matches/{match}/prints`.
#[derive(Deserialize)]
pub struct MatchPath {
    pub tournament: String,
    #[serde(rename = "match")]
    pub key: String,
}

/// `{tournament}`, `{match}` e `{game}` de `.../games/{game}/prints`.
#[derive(Deserialize)]
pub struct GamePath {
    pub tournament: String,
    #[serde(rename = "match")]
    pub key: String,
    pub game: i16,
}

/// `{tournament}`, `{match}`, `{game}` e `{print}` de
/// `.../games/{game}/prints/{print}`.
#[derive(Deserialize)]
pub struct PrintPath {
    pub tournament: String,
    #[serde(rename = "match")]
    pub key: String,
    pub game: i16,
    pub print: i64,
}
