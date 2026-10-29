//! Chave e resultados, mapa a mapa: ver a chave e uma partida, enviar um
//! mapa, confirmar ou contestar (`report`), e o organizador lançar ou
//! corrigir mapas e dar W.O. na série (`manage`) ou anular um mapa para ser
//! jogado de novo (`replay`).
//!
//! Cada mapa da série é enviado e confirmado sozinho (`games`); a série
//! fecha quando os mapas confirmados dão a maioria (regra do `engine`,
//! `series::games`), e só a série decidida entra na chave (`result`). A chave
//! nunca é gravada: sai do `replay` do `engine` com a configuração da fase,
//! os participantes pelo seed e as séries decididas, na ordem em que
//! passaram a valer (`stage`). Gerar e descartar a chave fica no início e no
//! desfazer do `lifecycle` (`start`, `discard`). Depois de toda mudança,
//! `settle` anula os mapas que ficaram depois da decisão, fecha as séries
//! decididas, dá os W.O. automáticos de quem desistiu ou saiu da comunidade
//! (`withdrawals`) e encerra ou reabre o torneio conforme a chave. A
//! confirmação por tempo é aplicada ao ler e agir (`due`), e o agendador do
//! tempo real a aplica nos torneios que ninguém leu (`confirm_all_due`). O que
//! muda na chave vai para o outbox pelo `settle` e pelo início (`changes`),
//! com os avisos de partida pronta, mapa enviado e contestado. A desistência,
//! que fica no `lifecycle`, chama `settle_withdrawals`. O lugar de quem vê na
//! chave e a próxima partida dele, para `/me/tournaments`, saem de `place`
//! (`mine`); o de outro inscrito, para o perfil do membro, de `place_of`. O
//! veto de mapas e campeões de cada partida, entre ela ficar pronta e o envio
//! de cada mapa, fica em `veto` (o relógio dos passos, `veto_clock`, é do
//! `realtime`).

mod actions;
mod announce;
mod changes;
mod db;
mod detail;
mod due;
mod games;
mod handlers;
mod history;
mod key;
mod manage;
mod mine;
mod outcome;
mod replay;
mod report;
mod result;
mod settle;
mod stage;
mod tasks;
mod types;
mod veto;
mod view;
mod withdrawals;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use due::{apply_due_confirmations, confirm_all_due};
pub use mine::{Place, place, place_of};
pub use stage::{discard, start};
pub use types::NextMatch;
pub use veto::clock as veto_clock;
pub use withdrawals::settle as settle_withdrawals;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::get_bracket))
        .routes(routes!(handlers::get_match))
        .routes(routes!(report::report))
        .routes(routes!(report::confirm))
        .routes(routes!(report::contest))
        .routes(routes!(manage::put_game))
        .routes(routes!(manage::set_result))
        .routes(routes!(replay::replay))
        .routes(routes!(veto::handlers::veto_ready))
        .routes(routes!(veto::handlers::veto_begin))
        .routes(routes!(veto::handlers::veto_step))
        .routes(routes!(veto::handlers::veto_undo))
        .routes(routes!(veto::handlers::veto_undo_request))
        .routes(routes!(veto::handlers::veto_undo_answer))
}
