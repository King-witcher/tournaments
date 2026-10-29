//! Testes de rota da API, um módulo por domínio (os mesmos de `src/`). Ficam
//! num binário só: cada arquivo solto em `tests/` vira um binário que linka a
//! crate inteira de novo. O banco de cada teste vem de `db::test_db`.

mod auth;
mod common;
mod db;
mod docs;
mod health;
mod invites;
mod lifecycle;
mod maps;
mod matches;
mod members;
mod my_tournaments;
mod organizing;
mod participants;
mod ratings;
mod realtime;
mod schema;
mod tenants;
mod tournaments;
