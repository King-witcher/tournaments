# Torneios de Quake Champions

Site multi-tenant para organizar torneios de Quake Champions (duelo, FFA, FFA por pontos/frags, 2v2, 3v3), em português e espanhol. Decisões e motivos: `docs/decisions.md`.

## Estrutura

```
backend/                cargo workspace (organização: backend/CLAUDE.md)
  crates/engine/        formatos, pontuação, regras de campeões, rating (puro, sem I/O)
  crates/api/src/       Axum: um módulo por domínio (auth/, tenants/, members/, invites/, tournaments/) + shared/
  crates/api/tests/api/ testes de rota, um arquivo por domínio, num binário só
  crates/migrate/       binário que só aplica as migrations (sem macros de query)
  migrations/           SQL do sqlx
web/                    Next.js, só apresentação (organização: web/CLAUDE.md)
  src/app/              rotas, só compõem
  src/features/         um domínio por pasta, os mesmos da API
  src/shared/           ui/, hooks/, api/, lib/ genéricos
docs/decisions.md
```

O código é organizado por domínio dos dois lados, com `shared/` para o que é genérico. Antes de criar arquivo, módulo ou componente, siga `backend/CLAUDE.md` ou `web/CLAUDE.md` (o Claude Code carrega cada um ao abrir arquivos da pasta): onde cada coisa vai, nomes e quando quebrar um arquivo que cresceu.

## Comandos

Backend (em `backend/`):
- `bun scripts/dev.ts`: API com recarga automática (para, aplica migrations, recompila e sobe a cada `.rs`/`.sql`/`Cargo.toml`/`.env` salvo; respeita `CARGO_TARGET_DIR` e `API_ADDR`)
- `cargo run -p api`: sobe a API em http://localhost:4000 (docs Scalar em `/docs`, OpenAPI em `/openapi.json`) e aplica as migrations
- `cargo test -p engine`: testes dos motores (rápidos, sem banco)
- `cargo test`: tudo, inclusive testes de API com `sqlx::test` (precisa do Postgres e do Redis locais)
- `cargo clippy --all-targets -- -D warnings` e `cargo fmt`
- Nova migration: criar `migrations/AAAAMMDDHHMMSS_nome.sql` à mão e rodar `cargo run -p migrate` **antes** de compilar código com `query!` que use a tabela nova (as macros conferem o SQL contra o banco local na compilação). Não há sqlx-cli; não instale globalmente.
- `bun scripts/sqlx-prepare.ts`: regenera `.sqlx/`, os dados das queries para compilar sem banco (`SQLX_OFFLINE=true`, como no Render). Rode e faça commit do `.sqlx/` sempre que criar ou mudar um `query!`, senão o deploy falha.
- `.cargo/config.toml` define o `DATABASE_URL` do banco local para as macros e, no Windows, o linker `rust-lld`.

Frontend (em `web/`), sempre com Bun (nada de npm/pnpm):
- `bun run dev`, `bun run build`
- `bun run lint` (Biome) e `bun run format` (Biome com --write). Ainda não há testes de frontend.
- `bun run gen:api`: regenera `src/shared/api/schema.d.ts` a partir do OpenAPI (com a API rodando)

Variáveis de ambiente: `backend/.env` e `web/.env.local`, a partir dos `.env.example`. Setup completo do banco e do Redis locais no `README.md`.

## Regras

- `engine` não depende de axum, sqlx, tokio nem do relógio. Aleatoriedade entra por seed/RNG passado como parâmetro.
- Sem ORM: SQL escrito à mão com `sqlx::query!`/`query_as!`. Migrations em `backend/migrations/`.
- Todo acesso a dados de tenant recebe um `TenantId` tipado. Nunca filtre tenant com string solta.
- Rota que exige login recebe o extractor `CurrentUser` (`crates/api/src/auth/current_user.rs`); rota pública que muda para quem está logado, `Option<CurrentUser>`. Rota `/tenants/{slug}/...` recebe `TenantMember<Player | Organizer | Owner>` (`tenants/access.rs`), que confere participação e papel mínimo e entrega o `TenantId`.
- Erros saem por `ApiError` (`shared/error.rs`) como `{"error": "<código>"}`. Corpo e caminho entram por `JsonBody`/`PathParams` (`shared/extract.rs`), não por `Json`/`Path` do axum, para a rejeição também sair como código.
- Ids: chave interna `bigint` identity; identificador público é slug; segredo é token aleatório com só o hash guardado, no banco ou no Redis (ver "Ids" em `docs/decisions.md`).
- Estado de formato no `engine` é reconstruído por `replay` (config + participantes + histórico); nunca desserializado direto.
- A API devolve códigos (`match.already_reported`), nunca texto para o usuário. Traduções ficam em `web/`. A exceção são os emails, que saem da API com o texto em pt e es (`auth/emails.rs`).
- Datas em UTC no banco.
- O Next nunca acessa o banco; só chama a API Rust.
- Supabase é só host de Postgres: não use Supabase Auth, Realtime, Storage nem supabase-js.
- Redis (Upstash grátis: 256 MB e 500 mil comandos por mês) só para dado efêmero com TTL: hoje, sessões, tentativas de senha, cadastros esperando o link e links por email (`auth/kv/`) e limites de envio de email (`mail/kv.rs`). Cada comando conta na cota: nada de Redis novo no caminho de toda requisição, nem cache ou lock sem medir antes (ver "Redis" em `docs/decisions.md`). Localmente, sempre no database 1 (os outros são de outros projetos do usuário; no Upstash, o 0, o único que há), e chave sempre por `Redis::key` (`shared/redis.rs`), que põe o prefixo do ambiente.
- Storage pela API S3 genérica, provedor por variável de ambiente.
- Email pelo Resend, por API HTTP (o Render free bloqueia SMTP), atrás da trait `Mailer` (`mail/`). Todo envio passa por `Mail::reserve`, que aplica os limites (60 s por endereço, teto do dia abaixo dos 100 do plano grátis).
- Nenhum segredo no repositório (ele vai ficar público).

## Como trabalhar aqui

- Ordem do trabalho e próximo passo: `docs/roadmap.md`. Ao terminar um incremento, ajuste a linha da área em "Feito" (uma linha por área; o detalhe vai no commit) e tire de "Pendências" o que resolveu.
- Decisões de produto em aberto: pergunte ao usuário (ele prefere rodadas de perguntas com opções) em vez de decidir sozinho; registre a resposta em `docs/decisions.md`.
- Subagentes (`.claude/agents/`): `format-engine` para trabalho no `engine`, `test-runner` para rodar testes, `rules-reviewer` para revisar um formato. Passe a especificação completa no prompt, porque eles não veem esta conversa.

## Armadilhas (Windows)

- Enquanto a API roda, `target\debug\api.exe` fica travado e `cargo build`/`cargo test` falham com "Access denied". Peça para o usuário parar a API, ou compile com o `CARGO_TARGET_DIR` alternativo, sempre o mesmo: `%LOCALAPPDATA%\Temp\claude\target-api`, para as sessões no checkout principal. Não crie um por sessão no scratchpad: cada um compila do zero e ocupa 3 a 5 GB. Se der "Access denied" nele, é outra sessão rodando a API de lá; use `target-api-2`, também fixo. **Worktree não usa o `target-api`**: compila no próprio `backend/target` (o `/worktree` já faz assim, e o `clean` apaga junto), porque o cargo dá o mesmo nome aos artefatos em todo worktree e, num target compartilhado, um worktree roda o binário de teste compilado de outro. Com o watcher (`bun scripts/dev.ts`), cada arquivo que você salva reinicia a API do usuário, e uma API sem reinício continua rodando código velho: depois de mudar schema ou rotas, confira se ela foi reiniciada.
- O usuário costuma deixar a API (4000) e o `bun run dev` (4001) rodando. Nunca encerre processos que você não iniciou; confira o dono com `Get-NetTCPConnection -LocalPort <porta>` e o horário de início.
- Só um `next dev` por pasta. Para testar o front com o do usuário rodando: `bun run build` e `bunx next start -p 4003`.
- Parar uma tarefa em segundo plano pode deixar o processo filho (`api.exe`, `node`) vivo. Depois de parar, confira as portas e encerre só o que for seu.
- Para regenerar os tipos com a API do usuário rodando código antigo: suba outra instância com `API_ADDR=127.0.0.1:4002` a partir do `CARGO_TARGET_DIR` alternativo e rode, em `web/`, `bunx openapi-typescript http://localhost:4002/openapi.json -o src/shared/api/schema.d.ts`.
