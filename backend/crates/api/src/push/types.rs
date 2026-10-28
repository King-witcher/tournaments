use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::shared::Locale;

#[derive(Serialize, ToSchema)]
pub struct PushKey {
    /// Chave pública VAPID em base64url: o `applicationServerKey` de
    /// `pushManager.subscribe`.
    pub public_key: String,
}

/// A inscrição do navegador, como `PushSubscription.toJSON()` entrega (sem o
/// `expirationTime`), e o idioma dos avisos.
#[derive(Deserialize, ToSchema)]
pub struct PutSubscription {
    /// URL `https` do serviço de push do navegador.
    pub endpoint: String,
    pub keys: SubscriptionKeys,
    /// Idioma do texto dos avisos (o da página ao se inscrever).
    pub locale: Locale,
}

#[derive(Deserialize, ToSchema)]
pub struct SubscriptionKeys {
    /// Chave pública ECDH do navegador (base64url).
    pub p256dh: String,
    /// Segredo de autenticação (base64url).
    pub auth: String,
}

#[derive(Deserialize, ToSchema)]
pub struct DeleteSubscription {
    pub endpoint: String,
}
