use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::shared::ImageType;

/// O que o front precisa saber do logo e da capa antes de oferecer o envio.
#[derive(Serialize, ToSchema)]
pub struct ImageSettings {
    /// O bucket público está configurado. Sem ele, o envio de logo e de capa
    /// some da interface.
    pub enabled: bool,
    /// Tamanho máximo de um arquivo, em bytes.
    #[schema(example = 1048576)]
    pub max_bytes: u32,
    pub content_types: Vec<ImageType>,
    /// O começo da URL de toda imagem que as regras aceitam
    /// (`<CDN>/rules/`); o Markdown só desenha imagem com ele. Nulo sem o
    /// bucket público: nenhuma imagem vale nas regras.
    #[schema(example = "https://img.telefrag.com.br/rules/")]
    pub rules_image_prefix: Option<String>,
}

/// Pedido da URL de upload de um logo, de uma capa, de uma foto ou de uma
/// imagem das regras.
#[derive(Deserialize, ToSchema)]
pub struct NewImageUpload {
    pub content_type: ImageType,
    /// Bytes, de 1 a `max_bytes`.
    pub size: u32,
}

#[derive(Serialize, ToSchema)]
pub struct ImageUpload {
    pub id: i64,
    /// URL assinada para o PUT direto ao bucket.
    pub url: String,
    /// O `Content-Type` que o PUT leva.
    pub content_type: ImageType,
    /// O `Cache-Control` que o PUT leva (o bucket grava no objeto).
    pub cache_control: String,
    pub expires_at: DateTime<Utc>,
}

/// A imagem registrada (logo, capa, foto ou imagem das regras).
#[derive(Serialize, ToSchema)]
pub struct PublicImage {
    /// A imagem no CDN.
    pub url: String,
}

/// `{upload}` de `/tenants/{slug}/logo/uploads/{upload}/attach`.
#[derive(Deserialize)]
pub struct LogoUploadPath {
    pub upload: i64,
}

/// `{tournament}` de `.../tournaments/{tournament}/cover` e de
/// `.../tournaments/{tournament}/rules/images/uploads`.
#[derive(Deserialize)]
pub struct TournamentImagePath {
    pub tournament: String,
}

/// `{tournament}` e `{upload}` de `.../cover/uploads/{upload}/attach` e de
/// `.../rules/images/uploads/{upload}/attach`.
#[derive(Deserialize)]
pub struct TournamentUploadPath {
    pub tournament: String,
    pub upload: i64,
}

/// `{upload}` de `/me/avatar/uploads/{upload}/attach`.
#[derive(Deserialize)]
pub struct AvatarUploadPath {
    pub upload: i64,
}
