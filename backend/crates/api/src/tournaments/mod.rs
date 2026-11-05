//! Torneios de um tenant: criar (rascunho), listar, ver, editar (o endereço, só
//! no rascunho), publicar, excluir (na hora, sem aviso; depois de começar, só o
//! owner) e traduzir os textos. Formato e regra de campeões passam
//! pela checagem do `engine` ao criar, ao mudar e ao publicar (e, com os
//! participantes reais, ao iniciar: `Setup::check_entrants`). Também decide
//! quem vê cada torneio (`visibility`: só o rascunho some para os
//! jogadores) e quem pode se inscrever pelo botão (`admission`), que `participants`, `invites` e
//! `matches` usam, e guarda a data de início e a janela de check-in
//! (`schedule`), com a abertura automática do check-in ao ler, e a janela de
//! contestação de resultado (`contest`), a configuração do veto das partidas
//! (`veto`: modo do mapa, pool, campeões, tempo por passo), quem age pelo time
//! (`team_rules`) e o screenshot obrigatório (`screenshots`). As transições manuais depois de
//! publicar ficam no domínio `lifecycle`; encerrar e reabrir conforme a chave,
//! em `matches` (por `sync_finished`). O que o torneio emite no outbox fica em
//! `changes` (todo domínio que mexe num torneio chama `changed`), e o que o
//! agendador do tempo real faz sozinho (abrir o check-in, avisar a hora de
//! iniciar), em `scheduled`.

mod admission;
mod changes;
mod contest;
mod db;
mod edit;
mod format;
mod handlers;
mod schedule;
mod scheduled;
mod screenshots;
mod setup;
mod slug;
mod status;
mod team_rules;
mod texts;
mod translations;
mod types;
mod veto;
mod view;
mod visibility;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use admission::Admission;
pub use changes::{changed, changed_with, check_in_opened};
pub use db::{
    Address, Listed, StageRow, StoredTournament, TournamentId, champion_pool, find,
    find_for_update, list_in, set_status, slugs_for_member, sync_finished,
};
pub use scheduled::{open_due_check_ins, remind_due_starts};
pub use setup::TournamentMode;
pub use status::TournamentStatus;
pub use team_rules::{TeamAction, TeamPicker, TeamRules};
pub use types::{
    NameTranslation, NoRepeatWho, StageFormat, Tournament, TournamentPath, TournamentSummary,
};
pub use veto::{
    MapMode, PickOrder, PoolMap, VetoInput, VetoOrder, VetoSetup, resolve as resolve_veto,
};
pub use visibility::{find_visible, find_visible_for_update};

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
