# Torneios de Quake Champions

Site multi-tenant para organizar torneios de Quake Champions (duelo, FFA, FFA por pontos/frags, 2v2, 3v3), em português e espanhol. Decisões e motivos, por área: `docs/decisions.md` (índice) e `docs/decisions/`.

## Estrutura

- `backend/`: cargo workspace (`engine` puro, `api` Axum com um módulo por domínio, `migrate`, `migrations/`). Organização em `backend/CLAUDE.md`.
- `web/`: Next.js, só apresentação, uma pasta por domínio em `src/features/`, os mesmos da API. Organização em `web/CLAUDE.md`.
- `docs/`: `decisions.md` e `decisions/` (o que foi decidido e por quê), `roadmap.md` (feito e pendências), `design.md` (alvo das páginas).

O código é organizado por domínio dos dois lados, com `shared/` para o genérico. Antes de criar arquivo, módulo ou componente, siga `backend/CLAUDE.md` ou `web/CLAUDE.md` (o Claude Code carrega cada um ao abrir arquivos da pasta).

## Comandos

Backend (em `backend/`):
- `bun scripts/dev.ts`: API com recarga automática (a cada `.rs`/`.sql`/`Cargo.toml`/`.env` salvo: para, aplica migrations, recompila e sobe; respeita `CARGO_TARGET_DIR` e `API_ADDR`). `cargo run -p api` sobe uma vez, em http://localhost:4000 (Scalar em `/docs`, OpenAPI em `/openapi.json`), e aplica as migrations.
- `cargo test -p engine`: motores, rápidos, sem banco. `cargo test`: tudo, inclusive rotas (Postgres e Redis locais; dividem 8 bancos reaproveitados, e a suíte de outra sessão espera esta terminar).
- `cargo clippy --all-targets -- -D warnings` e `cargo fmt`.
- Nova migration: criar `migrations/AAAAMMDDHHMMSS_nome.sql` à mão e rodar `cargo run -p migrate` **antes** de compilar código com `query!` que use a tabela nova (as macros conferem o SQL contra o banco local). Não há sqlx-cli; não instale globalmente. **Migration aplicada não muda, nem em comentário**: o sqlx confere o checksum do arquivo inteiro e staging/produção falham ao subir.
- `bun scripts/sqlx-prepare.ts`: regenera `.sqlx/` (compila sem banco, `SQLX_OFFLINE=true`, como no Render). Rode e faça commit sempre que criar ou mudar um `query!`, senão o deploy falha.
- `.cargo/config.toml` define o `DATABASE_URL` local para as macros e, no Windows, o linker `rust-lld`.

Frontend (em `web/`), sempre Bun (nada de npm/pnpm): `bun run dev`, `bun run build`, `bun run lint` e `bun run format` (Biome; ainda não há testes de frontend), `bun run gen:api` (regenera `src/shared/api/schema.d.ts` do OpenAPI, com a API rodando).

Variáveis de ambiente: `backend/.env` e `web/.env.local`, a partir dos `.env.example`. Setup do banco e do Redis locais no `README.md`.

## Regras

- `engine` não depende de axum, sqlx, tokio nem do relógio. Aleatoriedade entra por seed/RNG passado como parâmetro. Estado de formato é reconstruído por `replay` (config + participantes + histórico), nunca desserializado direto.
- Sem ORM: SQL à mão com `sqlx::query!`/`query_as!`.
- Todo acesso a dados de tenant recebe um `TenantId` tipado; nunca filtre tenant com string solta.
- Rota que exige login recebe `CurrentUser` (`auth/current_user.rs`); rota pública que muda para quem está logado, `Option<CurrentUser>`. Rota `/tenants/{slug}/...` recebe `TenantMember<Player | Organizer | Owner>` (`tenants/access.rs`), que confere participação e papel mínimo e entrega o `TenantId`.
- Erros saem por `ApiError` (`shared/error.rs`) como `{"error": "<código>"}`. Corpo e caminho entram por `JsonBody`/`PathParams` (`shared/extract.rs`), não por `Json`/`Path` do axum, para a rejeição também sair como código.
- A API devolve códigos (`match.already_reported`), nunca texto para o usuário; as traduções ficam em `web/`. Exceções: emails (`auth/emails/`, layout em `mail/layout.rs`) e avisos do Discord (`discord/catalog.rs`), em pt e es.
- Ids: chave interna `bigint` identity; identificador público é slug; segredo é token aleatório com só o hash guardado, no banco ou no Redis (`docs/decisions/stack.md`, "Ids"). Datas em UTC no banco.
- O Next nunca acessa o banco. Supabase é só host de Postgres: nada de Supabase Auth, Realtime, Storage nem supabase-js.
- Redis (Upstash grátis: 256 MB e 500 mil comandos por mês) só para dado efêmero com TTL (`auth/kv/`, `mail/kv.rs`). Cada comando conta na cota: nada de Redis novo no caminho de toda requisição, nem cache ou lock sem medir antes (`docs/decisions/stack.md`, "Redis"). Localmente, sempre no database 1 (os outros são de outros projetos do usuário; no Upstash, o 0), e chave sempre por `Redis::key` (`shared/redis.rs`), que põe o prefixo do ambiente.
- Storage pela API S3 genérica, provedor por variável de ambiente.
- Email pelo Resend, por API HTTP (o Render free bloqueia SMTP), atrás da trait `Mailer` (`mail/`). Todo envio passa por `Mail::reserve` (60 s por endereço, teto do dia abaixo dos 100 do plano grátis); os avisos de segurança, por `Mail::reserve_notice`, só pelo teto do dia.
- Nenhum segredo no repositório (ele vai ficar público).

## Como trabalhar aqui

- Próximo passo e pendências: `docs/roadmap.md`. Ao terminar um incremento, ajuste a linha da área em "Feito" (uma linha por área; o detalhe vai no commit) e tire de "Pendências" o que resolveu; cada pendência é curta: o que falta e onde.
- Decisão de produto em aberto: pergunte ao usuário (ele prefere rodadas de perguntas com opções) em vez de decidir sozinho, e registre a resposta no arquivo da área em `docs/decisions/` (poucas linhas: decisão, motivo, número que importa).
- Subagentes (`.claude/agents/`): `format-engine` (trabalho no `engine`), `rules-reviewer` (revisar um formato), `test-runner` (rodar testes), `investigator` (causa de um erro, só leitura). Passe a especificação completa no prompt: eles não veem esta conversa.

## Armadilhas (Windows)

- Enquanto a API roda, `target\debug\api.exe` fica travado e `cargo build`/`cargo test` falham com "Access denied". Peça ao usuário para parar a API, ou compile com o `CARGO_TARGET_DIR` alternativo, sempre o mesmo: `%LOCALAPPDATA%\Temp\claude\target-api`, para as sessões no checkout principal (não crie um por sessão: cada um compila do zero e ocupa 3 a 5 GB). "Access denied" nele é outra sessão rodando a API de lá: use `target-api-2`, também fixo. **Worktree não usa o `target-api`**: compila no próprio `backend/target` (o `/worktree` já faz assim e o `clean` apaga junto), porque o cargo dá o mesmo nome aos artefatos em todo worktree e, num target compartilhado, um worktree roda o binário de teste de outro.
- Com o watcher (`bun scripts/dev.ts`), cada arquivo que você salva reinicia a API do usuário, e uma API sem reinício continua rodando código velho: depois de mudar schema ou rotas, confira se ela reiniciou.
- O usuário costuma deixar a API (4000) e o `bun run dev` (4001) rodando. Nunca encerre processos que você não iniciou; confira o dono com `Get-NetTCPConnection -LocalPort <porta>` e o horário de início. Parar uma tarefa em segundo plano pode deixar o filho (`api.exe`, `node`) vivo: confira as portas e encerre só o que for seu.
- Só um `next dev` por pasta. Para testar o front com o do usuário rodando: `bun run build` e `bunx next start -p 4003`.
- Para regenerar os tipos com a API do usuário rodando código antigo: suba outra instância com `API_ADDR=127.0.0.1:4002` a partir do `CARGO_TARGET_DIR` alternativo e rode, em `web/`, `bunx openapi-typescript http://localhost:4002/openapi.json -o src/shared/api/schema.d.ts`.
