//! Torneios de um tenant: criar (rascunho), listar, ver, editar, publicar,
//! excluir rascunho e traduzir os textos. Formato e regra de campeões passam
//! pela checagem do `engine` ao criar, ao mudar e ao publicar. Também decide
//! quem vê cada torneio (`visibility`), que `participants` e `invites` usam,
//! e guarda a data de início e a janela de check-in (`schedule`), com a
//! abertura automática do check-in ao ler. As transições manuais depois de
//! publicar ficam no domínio `lifecycle`.

mod db;
mod edit;
mod format;
mod handlers;
mod schedule;
mod setup;
mod slug;
mod status;
mod texts;
mod translations;
mod types;
mod view;
mod visibility;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use db::{
    StoredTournament, TournamentId, find, find_for_update, mark_registration_reopened, set_status,
};
pub use status::TournamentStatus;
pub use types::{NameTranslation, Tournament, TournamentPath};
pub use visibility::{Visibility, find_visible, find_visible_for_update};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(
            handlers::create_tournament,
            handlers::list_tournaments
        ))
        .routes(routes!(
            handlers::get_tournament,
            handlers::update_tournament,
            handlers::delete_tournament
        ))
        .routes(routes!(handlers::publish_tournament))
        .routes(routes!(
            translations::put_translation,
            translations::delete_translation
        ))
}
