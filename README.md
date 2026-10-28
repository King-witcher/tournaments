# Torneios de Quake Champions

Site multi-tenant para organizar torneios de Quake Champions. API em Rust (Axum + sqlx), frontend em Next.js. Decisões do projeto em [`docs/decisions.md`](docs/decisions.md).

## Pré-requisitos

- PostgreSQL 18 local
- Rust (stable)
- Bun e Node 24 (o Bun instala e roda os scripts; o Next roda em Node, igual à Vercel)

## Banco local

O projeto usa um banco `tournaments` com um usuário `tournaments` que só acessa esse banco e entra sem senha a partir desta máquina. Para criar numa máquina nova, como `postgres`:

```sql
CREATE ROLE tournaments LOGIN NOSUPERUSER NOCREATEROLE CREATEDB NOREPLICATION;
CREATE DATABASE tournaments OWNER tournaments;
REVOKE ALL ON DATABASE tournaments FROM PUBLIC;
GRANT CONNECT, TEMPORARY ON DATABASE tournaments TO tournaments;
```

`CREATEDB` existe só para os testes: o `sqlx::test` cria um banco temporário `_sqlx_test_*` por teste.

No `pg_hba.conf`, **antes** das linhas `all all`, e depois `SELECT pg_reload_conf();`:

```
host    tournaments,/^_sqlx_test    tournaments    127.0.0.1/32    trust
host    tournaments,/^_sqlx_test    tournaments    ::1/128         trust
local   tournaments,/^_sqlx_test    tournaments                    trust
host    all                         tournaments    all             reject
local   all                         tournaments                    reject
```

As três primeiras liberam o usuário sem senha só no próprio banco e nos de teste; as duas últimas recusam o resto.

## Rodando

Na primeira vez:

```sh
cp backend/.env.example backend/.env
cp web/.env.example web/.env.local
cd web && bun install
```

Para o login, crie uma aplicação em https://discord.com/developers/applications, adicione o redirect `http://localhost:4000/auth/discord/callback` em **OAuth2 → Redirects** e coloque `DISCORD_CLIENT_ID` e `DISCORD_CLIENT_SECRET` no `backend/.env`. Sem eles, tudo funciona menos o login com Discord (o login com nome de usuário e senha não depende dele).

Depois, em um terminal, a API com recarga automática: a cada `.rs`, `.sql`, `Cargo.toml` ou `.env` salvo, ela para, aplica as migrations, recompila e sobe de novo. Se a compilação falhar, espera a próxima mudança.

```sh
cd backend
bun scripts/dev.ts
```

Sem recarga, `cargo run -p api` sobe uma vez (e também aplica as migrations).

- API: http://localhost:4000
- Docs (Scalar): http://localhost:4000/docs

Em outro terminal, o frontend:

```sh
cd web
bun run dev
```

- Site: http://localhost:4001 (redireciona para `/pt` ou `/es` conforme o navegador)

## Dia a dia

| O quê | Onde | Comando |
|---|---|---|
| API com recarga automática | `backend/` | `bun scripts/dev.ts` |
| Testes do engine (rápidos, sem banco) | `backend/` | `cargo test -p engine` |
| Todos os testes do backend | `backend/` | `cargo test` |
| Lint do backend | `backend/` | `cargo clippy --all-targets -- -D warnings` |
| Nova migration | `backend/` | criar `migrations/AAAAMMDDHHMMSS_nome.sql` e rodar `cargo run -p migrate` |
| Regenerar tipos da API no front (API rodando) | `web/` | `bun run gen:api` |
| Lint / formatar o front (Biome) | `web/` | `bun run lint` / `bun run format` |
| Build do front | `web/` | `bun run build` |
