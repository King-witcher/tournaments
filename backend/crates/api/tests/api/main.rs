//! Testes de rota da API, um módulo por domínio (os mesmos de `src/`). Ficam
//! num binário só: cada arquivo solto em `tests/` vira um binário que linka a
//! crate inteira de novo.

mod auth;
mod common;
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
