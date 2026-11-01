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
}

/// Pedido da URL de upload de um logo ou de uma capa.
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

/// O logo ou a capa registrados.
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

/// `{tournament}` de `.../tournaments/{tournament}/cover`.
#[derive(Deserialize)]
pub struct CoverPath {
    pub tournament: String,
}

/// `{tournament}` e `{upload}` de `.../cover/uploads/{upload}/attach`.
#[derive(Deserialize)]
pub struct CoverUploadPath {
    pub tournament: String,
    pub upload: i64,
}
