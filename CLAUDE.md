# Torneios de Quake Champions

Site multi-tenant para organizar torneios de Quake Champions (duelo, FFA, FFA por pontos/frags, 2v2, 3v3), em português e espanhol. Decisões e motivos: `docs/decisions.md`.

## Estrutura

```
backend/                cargo workspace
  crates/engine/        formatos, pontuação, regras de campeões, rating (puro, sem I/O)
  crates/api/           Axum: rotas, WebSocket, auth, acesso ao banco
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
- Nova migration: criar `migrations/AAAAMMDDHHMMSS_nome.sql` à mão (não há sqlx-cli instalado; não instale globalmente)

Frontend (em `web/`), sempre com Bun (nada de npm/pnpm):
- `bun run dev`, `bun run build`
- `bun run lint` (Biome) e `bun run format` (Biome com --write). Ainda não há testes de frontend.
- `bun run gen:api`: regenera `src/lib/api/schema.d.ts` a partir do OpenAPI (com a API rodando)

Variáveis de ambiente: `backend/.env` e `web/.env.local`, a partir dos `.env.example`. Setup completo do banco local no `README.md`.

## Regras

- `engine` não depende de axum, sqlx, tokio nem do relógio. Aleatoriedade entra por seed/RNG passado como parâmetro.
- Sem ORM: SQL escrito à mão com `sqlx::query!`/`query_as!`. Migrations em `backend/migrations/`.
- Todo acesso a dados de tenant recebe um `TenantId` tipado. Nunca filtre tenant com string solta.
- A API devolve códigos (`match.already_reported`), nunca texto para o usuário. Traduções ficam em `web/`.
- Datas em UTC no banco.
- O Next nunca acessa o banco; só chama a API Rust.
- Supabase é só host de Postgres: não use Supabase Auth, Realtime, Storage nem supabase-js.
- Storage pela API S3 genérica, provedor por variável de ambiente.
- Nenhum segredo no repositório (ele vai ficar público).

## Subagentes (`.claude/agents/`)

- `test-runner`: roda testes e devolve só as falhas.
- `format-engine`: implementa ou altera formatos, pontuação, regras de campeões e rating no `engine`.
- `rules-reviewer`: revisa casos de borda de um formato (só leitura).
