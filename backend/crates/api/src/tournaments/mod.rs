//! Torneios de um tenant: criar (rascunho), listar, ver, editar (o endereço, só
//! no rascunho), publicar e voltar para rascunho (sem nenhum inscrito), excluir
//! (na hora, sem aviso; depois de começar, só o owner) e traduzir os textos. Formato e regra de campeões passam
//! pela checagem do `engine` ao criar, ao mudar e ao publicar (e, com os
//! participantes reais, ao iniciar: `Setup::check_entrants`). Também decide
//! quem vê cada torneio (`visibility`: só o rascunho some para os
//! jogadores) e quem pode se inscrever pelo botão (`admission`), que `participants`, `invites` e
//! `matches` usam, e guarda a data de início, só informativa, e o check-in
//! ligado ou desligado (`schedule`), a janela de contestação de resultado
//! (`contest`), a configuração do veto das partidas
//! (`veto`: modo do mapa, pool, campeões, tempo por passo), quem age pelo time
//! (`team_rules`) e o screenshot obrigatório (`screenshots`). As imagens das
//! regras (`rules_images`) são as que `images` registra na comunidade; ao
//! gravar as regras, o texto só aceita essas, e o banco guarda quais textos
//! usam cada uma. As transições manuais depois de
//! publicar ficam no domínio `lifecycle`; encerrar e reabrir conforme a chave,
//! em `matches` (por `sync_finished`). Cada fase tem a configuração (o
//! formato, o corte e o check-in, que mudam até ela começar: o PATCH muda as
//! fases no lugar, e tirar as que faltam com a atual terminada encerra) e o
//! andamento (`StageProgress`: início, check-in aberto, fim), que `matches` e
//! `stages` acertam (`db::stages`). O PATCH fica em `update`, com a rota no
//! `lifecycle`: o pódio do encerramento por ele é da chave. O que o torneio emite no outbox fica em
//! `changes` (todo domínio que mexe num torneio chama `changed`).

mod admission;
mod changes;
mod contest;
mod db;
mod edit;
mod format;
mod handlers;
mod rules_images;
mod schedule;
mod screenshots;
mod setup;
mod slug;
mod status;
mod team_rules;
mod texts;
mod translations;
mod types;
mod update;
mod veto;
mod view;
mod visibility;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use admission::Admission;
pub use changes::{changed, changed_with, check_in_opened};
pub use db::{
    Address, Listed, StageProgress, StageRow, StoredTournament, TournamentId, champion_pool,
    delete_unused_rules_images, find, find_for_update, list_in, mark_stage_finished,
    mark_stage_started, open_stage_check_in, register_rules_image, reset_stages, set_status,
    slugs_for_member, sync_finished, unmark_stage_started, unused_rules_images,
};
pub use rules_images::cdn as rules_images_cdn;
pub use setup::TournamentMode;
pub use status::TournamentStatus;
pub use team_rules::{TeamAction, TeamPicker, TeamRules};
pub use types::{
    NameTranslation, NoRepeatWho, StageFormat, StageStatus, Tournament, TournamentPath,
    TournamentSummary, UpdateTournament,
};
pub use update::update;
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
            handlers::delete_tournament
        ))
        .routes(routes!(
            handlers::publish_tournament,
            handlers::unpublish_tournament
        ))
        .routes(routes!(
            translations::put_translation,
            translations::delete_translation
        ))
}
