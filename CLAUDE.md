# Torneios de Quake Champions

Site multi-tenant para organizar torneios de Quake Champions (duelo, FFA, FFA por pontos/frags, 2v2, 3v3), em português e espanhol. Decisões e motivos: `docs/decisions.md`.

## Estrutura

```
backend/                cargo workspace
  crates/engine/        formatos, pontuação, regras de campeões, rating (puro, sem I/O)
  crates/api/           Axum: rotas, WebSocket, auth, acesso ao banco
  crates/migrate/       binário que só aplica as migrations (sem macros de query)
  migrations/           SQL do sqlx
web/                    Next.js, só apresentação (next-intl: pt, es)
docs/decisions.md
```

## Comandos

Backend (em `backend/`):
- `cargo run -p api`: sobe a API em http://localhost:8080 (Swagger em `/docs`) e aplica as migrations
- `cargo test -p engine`: testes dos motores (rápidos, sem banco)
- `cargo test`: tudo, inclusive testes de API com `sqlx::test` (precisa do Postgres local)
- `cargo clippy --all-targets -- -D warnings` e `cargo fmt`
- Nova migration: criar `migrations/AAAAMMDDHHMMSS_nome.sql` à mão e rodar `cargo run -p migrate` **antes** de compilar código com `query!` que use a tabela nova (as macros conferem o SQL contra o banco local na compilação). Não há sqlx-cli; não instale globalmente.
- `.cargo/config.toml` define o `DATABASE_URL` do banco local para as macros.

Frontend (em `web/`), sempre com Bun (nada de npm/pnpm):
- `bun run dev`, `bun run build`
- `bun run lint` (Biome) e `bun run format` (Biome com --write). Ainda não há testes de frontend.
- `bun run gen:api`: regenera `src/lib/api/schema.d.ts` a partir do OpenAPI (com a API rodando)

Variáveis de ambiente: `backend/.env` e `web/.env.local`, a partir dos `.env.example`. Setup completo do banco local no `README.md`.

## Regras

- `engine` não depende de axum, sqlx, tokio nem do relógio. Aleatoriedade entra por seed/RNG passado como parâmetro.
- Sem ORM: SQL escrito à mão com `sqlx::query!`/`query_as!`. Migrations em `backend/migrations/`.
- Todo acesso a dados de tenant recebe um `TenantId` tipado. Nunca filtre tenant com string solta.
- Rota que exige login recebe o extractor `CurrentUser` (`crates/api/src/current_user.rs`). Erros saem por `ApiError` (`error.rs`) como `{"error": "<código>"}`.
- Estado de formato no `engine` é reconstruído por `replay` (config + participantes + histórico); nunca desserializado direto.
- A API devolve códigos (`match.already_reported`), nunca texto para o usuário. Traduções ficam em `web/`.
- Datas em UTC no banco.
- O Next nunca acessa o banco; só chama a API Rust.
- Supabase é só host de Postgres: não use Supabase Auth, Realtime, Storage nem supabase-js.
- Storage pela API S3 genérica, provedor por variável de ambiente.
- Nenhum segredo no repositório (ele vai ficar público).

## Como trabalhar aqui

- Ordem do trabalho e próximo passo: `docs/roadmap.md`. Atualize a seção "Feito" ao terminar um incremento.
- Decisões de produto em aberto: pergunte ao usuário (ele prefere rodadas de perguntas com opções) em vez de decidir sozinho; registre a resposta em `docs/decisions.md`.
- Use os subagentes abaixo para trabalho no `engine` e para rodar testes; passe a especificação completa no prompt, porque eles não veem esta conversa.

## Armadilhas (Windows)

- Enquanto a API roda, `target\debug\api.exe` fica travado e `cargo build`/`cargo test` falham com "Access denied". Peça para o usuário parar a API, ou compile com `CARGO_TARGET_DIR` temporário.
- O usuário costuma deixar a API (8080) e o `bun run dev` (3000) rodando. Nunca encerre processos que você não iniciou; confira o dono com `Get-NetTCPConnection -LocalPort <porta>` e o horário de início.
- Só um `next dev` por pasta. Para testar o front com o do usuário rodando: `bun run build` e `bunx next start -p 3001`.
- Parar uma tarefa em segundo plano pode deixar o processo filho (`api.exe`, `node`) vivo. Depois de parar, confira as portas e encerre só o que for seu.

## Subagentes (`.claude/agents/`)

- `test-runner`: roda testes e devolve só as falhas.
- `format-engine`: implementa ou altera formatos, pontuação, regras de campeões e rating no `engine`.
- `rules-reviewer`: revisa casos de borda de um formato (só leitura).
