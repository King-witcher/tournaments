# Roadmap

Ordem de trabalho combinada. Decisões e motivos ficam em `decisions.md`; aqui fica o que fazer e em que ordem.

## Feito

- Scaffold: workspace Rust (`engine`, `api`, `migrate`), Next 16 com next-intl (pt, es), Biome, Bun, banco local.
- `engine`: eliminação simples e dupla (`formats/elimination.rs`), seeding padrão, validação de melhor de N, checagem de pool de campeões (`champions.rs`). Revisado pelo `rules-reviewer`.
- `api`: login com Discord, sessão em cookie (só o hash no banco), `/me`, logout, extractor `CurrentUser`, `ApiError`.
- `web`: menu do usuário (entrar/sair), erros de login traduzidos, status da API.
- `backend/scripts/dev.ts`: API com recarga automática (Bun, sem ferramenta global).
- Ids: `users` e o resto em `bigint` identity (migrations reescritas antes de existir produção); ver "Ids" em `decisions.md`.
- **Tenants** ("comunidades"): tabelas `tenants`, `tenant_members` (papel + apelido) e `tenant_invites` (hash do token, 7 dias, revogável). Rotas para criar (limite de 3 como owner), listar os meus, ver o tenant, editar o próprio apelido, listar membros, owner trocar player ↔ organizer, gerar, listar e revogar convites, prévia pública do convite e aceitar. Extractor `TenantMember<Papel>` entrega o `TenantId`; `JsonBody`/`PathParams` fazem erro de corpo/caminho sair como código. Testes `sqlx::test` em `tests/tenants.rs`. Front: "Minhas comunidades" na home, `/[locale]/new`, `/[locale]/[tenant]` e `/[locale]/invite/[token]`.
- **Organização por domínio**: API em `auth/`, `tenants/`, `members/`, `invites/` e `shared/`, cada domínio com `mod.rs` (router), `handlers.rs`, `types.rs` e `db.rs` (todo o SQL); testes de rota num binário só (`tests/api/`). `engine`: `formats/elimination/` dividido em `build`, `resolve` e testes por assunto. Front: `src/features/<domínio>` + `src/shared/` (`ui`, `hooks`, `api`, `lib`), páginas só compondo, `useApiAction`/`callApi` para mutações. Diretrizes em `backend/CLAUDE.md` e `web/CLAUDE.md`.

## Padrões assumidos (configuráveis, confirmar com o usuário quando surgir)

- Melhor de N: um valor padrão + um para a decisiva (final na simples; grande final e reset na dupla). Ex.: md3 + md5.
- Grande final com reset: opção do organizador.
- Ban de campeão pelo capitão; pick por cada jogador. Timeout no veto: escolha aleatória, desfazível pelo organizador.
- Corrigir resultado = refazer o estado com `replay` e o histórico corrigido; correção que contradiz partida posterior é recusada.

## Próximo: duelo de ponta a ponta

Um incremento por vez, cada um com migration, rotas (OpenAPI), testes `sqlx::test`, tipos regenerados no front e tela mínima.

1. **Torneio de duelo**
   - `tournaments` (tenant, slug, idioma original, nome, descrição, regras, status: rascunho → inscrição → check-in → em andamento → encerrado), `tournament_translations` (opcional por idioma), `stages` (ordem, config do formato em JSON validado pelos tipos do `engine`).
   - Ao criar: validar o formato e, se houver regra de não repetição, rodar `champions::check_budget` com o `worst_case_games` do formato. Bloquear com o código de erro e os números.
2. **Inscrição, check-in e seeding**
   - `participants` (torneio, jogador do tenant, seed, check-in). Janela de check-in; quem não fizer sai antes da chave.
   - Seeding manual, aleatório (seed salva para reproduzir) ou por rating (quando existir).
   - Decidir se o apelido é único dentro do tenant (em aberto no `decisions.md`).
3. **Chave e resultados**
   - Gerar a chave com `EliminationBracket`; persistir só config, participantes e `match_reports` (histórico); reconstruir por `replay`.
   - Envio de resultado por um lado, janela de contestação, organizador resolve contestação.
   - Front: visualização da chave (simples e dupla), página da partida.

## Depois

- Membros: remover membro, sair do tenant, transferir a posse.

- Tempo real (WebSocket no Axum) para chave, check-in e partida.
- Veto de mapas e pick/ban de campeões ao vivo, com regra de não repetição aplicada.
- Prints do placar (S3 genérico, URL assinada, WebP, apagar 30 dias após o fim).
- Rating (Elo, configurável por tenant).
- Outros formatos: FFA por pontos/frags, 2v2/3v3 com formação de times, round robin, suíço.
- Deploy: Render (API), Vercel (web), Supabase (Postgres), CI no GitHub Actions com `cargo sqlx prepare` para build offline.
