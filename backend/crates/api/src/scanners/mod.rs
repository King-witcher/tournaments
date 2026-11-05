//! Scanners de vulnerabilidade: quem pede o que este site nunca serviu
//! (`/.env`, `/wp-login.php`) tem o IP banido, e dali em diante toda rota responde
//! o mesmo 404 vazio de uma rota que não existe, sem nada que diga que houve ban.
//! O front faz o mesmo no proxy dele e divide a lista por `/internal/bans`
//! (`web/src/features/scanners`). Sem `INTERNAL_API_SECRET`, fica tudo desligado:
//! sem o segredo, o servidor do Next não se identifica e poderia ser banido.
//! Motivos em `docs/decisions/stack.md`, "Scanners".
//!
//! - `traps.rs`: os caminhos que denunciam e quem nunca é banido por eles.
//! - `bans.rs`: os bans em memória e o segredo do servidor do Next.
//! - `kv.rs`: a cópia no Redis, para sobreviverem a um boot.
//! - `handlers.rs`: o middleware, a rota que não existe e a rota interna.

mod bans;
mod handlers;
mod kv;
mod traps;

use utoipa_axum::router::OpenApiRouter;

use crate::AppState;

pub use bans::{Bans, SECRET_HEADER};
pub use handlers::{no_routes, refuse_banned};

/// Fora do OpenAPI: `/internal/bans` não é da API pública.
pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .route(
            "/internal/bans",
            axum::routing::any(handlers::internal_bans),
        )
        .fallback(handlers::unknown_route)
}
