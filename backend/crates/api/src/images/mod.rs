//! Imagens públicas: o logo da comunidade (quadrado, do owner), a capa do
//! torneio (2:1, da organização) e a foto de perfil (quadrada, da própria
//! pessoa, em `/me/avatar`), no bucket público que o CDN serve. O banco
//! guarda só o nome do objeto (`tenants.logo_key`, `tournaments.cover_key`,
//! `users.avatar_key`); as respostas de `tenants`, `tournaments`,
//! `my_tournaments` e `invites` montam a URL com `shared::public_url`, e as
//! que mostram pessoas, com `auth::avatar_url` (a foto vale mais que o avatar
//! do Discord). Como nos screenshots, o navegador
//! manda o arquivo direto ao bucket: pede a URL assinada (`.../uploads`, que
//! grava o upload pendente e conta os pedidos da pessoa, em `upload`), faz o
//! PUT (uma vez só por URL, com o `Cache-Control` de um ano) e registra
//! (`.../attach`, que confere tipo e tamanho por HEAD e grava o logo ou a
//! capa). Nenhum objeto fica órfão: o banco põe na fila de exclusão todo
//! objeto que deixa de ser usado (trocado, removido, de upload abandonado, da
//! comunidade ou do torneio apagados), e a rotina diária (`sweep`, chamada
//! pelo agendador do tempo real) esvazia a fila. Sem o bucket público
//! configurado, o envio fica desligado (`GET /images/settings`). Fica acima
//! de `tenants` e `tournaments`.

mod db;
mod handlers;
mod object;
mod sweep;
mod types;
mod upload;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use sweep::{Sweep, sweep, sweep_due};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::get_image_settings))
        .routes(routes!(handlers::create_logo_upload))
        .routes(routes!(handlers::attach_logo))
        .routes(routes!(handlers::remove_logo))
        .routes(routes!(handlers::create_cover_upload))
        .routes(routes!(handlers::attach_cover))
        .routes(routes!(handlers::remove_cover))
        .routes(routes!(handlers::create_avatar_upload))
        .routes(routes!(handlers::attach_avatar))
        .routes(routes!(handlers::remove_avatar))
}
