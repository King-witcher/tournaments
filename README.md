# Torneios de Quake Champions

Site multi-tenant para organizar torneios de Quake Champions. API em Rust (Axum + sqlx), frontend em Next.js. Decisões do projeto em [`docs/decisions.md`](docs/decisions.md).

## Pré-requisitos

- PostgreSQL 18 local
- Redis local (no Windows, o [Memurai](https://www.memurai.com/), compatível; ou `docker run -d -p 6379:6379 redis`)
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

## Redis local

Sessões e tentativas de senha ficam no Redis, sempre no database 0 (`redis://127.0.0.1:6379/0` no `.env.example`); os outros databases do servidor ficam para outros projetos. Dentro do 0, a API põe `REDIS_PREFIX` (padrão `tournaments:`) na frente de toda chave, o que separa os ambientes deste projeto (um worktree com outro prefixo não vê as sessões do checkout principal). Os testes usam o mesmo database (ou o de `REDIS_URL`), com um prefixo `test:` por execução e por teste; as chaves de execuções com mais de uma hora saem na execução seguinte.

## Rodando

Na primeira vez:

```sh
cp backend/.env.example backend/.env
cp web/.env.example web/.env.local
cd web && bun install
```

Para o login, crie uma aplicação em https://discord.com/developers/applications, adicione o redirect `http://localhost:4000/auth/discord/callback` em **OAuth2 → Redirects** e coloque `DISCORD_CLIENT_ID` e `DISCORD_CLIENT_SECRET` no `backend/.env`. Sem eles, tudo funciona menos o login com Discord (o login com nome de usuário e senha não depende dele).

Para os avisos fora do site (Web Push), gere as chaves VAPID com `bun scripts/vapid-keys.ts` (em `backend/`, sem instalar nada) e cole as duas linhas que ele imprime (`VAPID_PUBLIC_KEY` e `VAPID_PRIVATE_KEY`) no `backend/.env`; `VAPID_SUBJECT` (um `mailto:` ou `https:`) é opcional e, sem ele, vai a origem do front. Sem as chaves, tudo funciona e os avisos só chegam com o site aberto, pelo WebSocket. Em produção, as mesmas variáveis no Render: trocar o par depois faz cada navegador precisar ativar os avisos de novo.

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
| Todos os testes do backend (Postgres e Redis locais) | `backend/` | `cargo test` |
| Lint do backend | `backend/` | `cargo clippy --all-targets -- -D warnings` |
| Nova migration | `backend/` | criar `migrations/AAAAMMDDHHMMSS_nome.sql` e rodar `cargo run -p migrate` |
| Regenerar tipos da API no front (API rodando) | `web/` | `bun run gen:api` |
| Lint / formatar o front (Biome) | `web/` | `bun run lint` / `bun run format` |
| Build do front | `web/` | `bun run build` |
