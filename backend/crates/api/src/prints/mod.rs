//! Prints do placar: até 3 imagens por mapa da série, guardadas num bucket S3
//! (`shared::Storage`). O navegador manda o arquivo direto ao bucket: pede a
//! URL assinada (`POST .../games/{n}/prints`, que já grava a linha pendente),
//! faz o PUT e registra (`.../prints/{id}/attach`, que confere tipo e tamanho
//! no bucket por HEAD, em `upload`). O print vai num mapa enviado ou no
//! próximo, antes do envio: com print obrigatório no torneio, o envio do
//! jogador exige um (a regra fica no envio, em `matches`). Quem anexa e quem
//! remove, em `policy`; a partida (quem joga, os mapas) vem de `matches`. A
//! página da partida lê os prints com a URL de leitura assinada de cada um
//! (`view`). Nenhum objeto fica órfão: o banco põe na fila de exclusão o nome
//! de todo print apagado, por qualquer caminho, e a rotina diária (`sweep`,
//! chamada pelo agendador do tempo real) esvazia a fila e apaga os prints de
//! torneios encerrados há mais de 30 dias. Sem o bucket configurado, os prints
//! ficam desligados (`GET /prints/settings`). Fica acima de `matches`.

mod changes;
mod db;
mod handlers;
mod object;
mod policy;
mod sweep;
mod types;
mod upload;
mod view;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use sweep::{Sweep, sweep, sweep_due};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::get_print_settings))
        .routes(routes!(handlers::list))
        .routes(routes!(handlers::create_upload))
        .routes(routes!(handlers::attach))
        .routes(routes!(handlers::remove))
}
