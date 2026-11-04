//! Sala de veto: veto de mapas e draft de campeões avulsos, sem torneio nem
//! comunidade, para quem não usa o site para gerenciar a chave. Qualquer um
//! cria, sem login; o link é a credencial (um por jogador, ou um por time na sala
//! centrada no capitão, um de espectador e um de gestão, que mostra todos e pode
//! começar sem esperar). O veto é o mesmo da série de um torneio:
//! as regras são do `engine`, e o histórico, o passo, os sorteios e a visão vêm
//! de `matches::veto`; daqui é só o que muda sem o torneio.
//!
//! - `create`: criar, com os limites de quem cria (constantes lá).
//! - `handlers`: ler, entrar num lugar, "Pronto", começar sem esperar, um passo.
//! - `room`: a sala carregada e refeita (`Room`), o que grava (eventos e o
//!   próximo prazo) e os sorteios vencidos. A sala é o escopo da trava: toda
//!   escrita passa por `Room::lock` (`select ... for update`).
//! - `view`: a sala como cada link a vê, com a visão de `matches::veto`.
//! - `db`: as tabelas `veto_rooms`, `veto_room_seats`, `veto_room_maps` e
//!   `veto_room_events`.
//! - `live`: o canal em memória de cada sala, com os limites de conexão, e o
//!   que o espectador lê sem ir ao banco. O socket por token fica no `realtime`
//!   (`room_socket`), que está acima.
//! - `clock`: os prazos e a limpeza, chamados pelo `realtime` (o relógio dorme
//!   até o próximo prazo, e o agendador passa a cada 10 s). A coluna `due_at` diz
//!   o próximo sem refazer todas as salas.
//!
//! Não usa o outbox: mudança de sala não tem aviso nem push, e o `changed` sai
//! depois do commit pelo canal da própria sala (`docs/decisions/realtime.md`).

mod clock;
mod create;
mod db;
mod handlers;
mod live;
mod room;
mod types;
mod view;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use clock::{clock, delete_expired};
pub use db::{RoomAccess, access};
pub use live::{Change, Live, RoomSocket};
pub use types::VetoRoomPath;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(create::create_veto_room))
        .routes(routes!(handlers::get_veto_room))
        .routes(routes!(handlers::join_veto_room))
        .routes(routes!(handlers::veto_room_ready))
        .routes(routes!(handlers::veto_room_begin))
        .routes(routes!(handlers::veto_room_step))
}
