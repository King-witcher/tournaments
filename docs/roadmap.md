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
- **Torneio de duelo** (item 1 do duelo de ponta a ponta; decisões em "Torneios" no `decisions.md`):
  - Migration com `champions` (os 17 atuais), `tournaments` (vagas, `champion_rules` em JSON), `tournament_translations` e `tournament_stages` (`config` = `engine::formats::StageFormat` em JSON).
  - `engine`: `StageFormat` (validação, maior série e pior caso com N participantes), `ChampionRules` e `TournamentSetup::validate`, que junta vagas, fases e o orçamento de campeões. Tem teste de que o pior caso não diminui com mais participantes.
  - API (`tournaments/`): criar rascunho, listar (rascunho só para organizer/owner), ver, editar (textos até encerrar; formato até começar), publicar, excluir rascunho, criar e remover tradução. A checagem de campeões roda ao criar, ao mudar a configuração e ao publicar. `ApiError` ganhou `params` (`{"error", "params": {"required", "available"}}`) e os erros do `engine` viram códigos em `shared/engine.rs`. `TournamentId` tipado só sai de consulta filtrada por tenant.
  - Front: seção de torneios na página do tenant, `/[tenant]/new`, `/[tenant]/[torneio]` (Markdown com `react-markdown`, formato, publicar e excluir) e `/[tenant]/[torneio]/edit` (formulário e tradução). Erros com números interpolados na tradução. `slugify` foi para `shared/lib`.
  - `RUST_TEST_THREADS=4` no `.cargo/config.toml`: com uma thread por núcleo, a criação dos bancos do `sqlx::test` dava `PoolTimedOut`.
- **Login com username e senha** (decisões em "Autenticação" no `decisions.md`):
  - Migration: provedor `password` em `user_identities` (com `password_hash`), username único sem diferenciar maiúsculas, um login por provedor por usuário e `password_failures` para o limite de tentativas.
  - API (`auth/`): `POST /auth/signup`, `POST /auth/login`, `GET /auth/logins`, `POST`/`PATCH`/`DELETE /auth/password`, `GET /auth/discord/link` (o callback do Discord agora também vincula) e `DELETE /auth/discord`. argon2id fora das threads do tokio (`password.rs`), limite de 5 erros em 15 min por username (`attempts.rs`), SQL dividido em `db/` (sessões, logins, tentativas). O `argon2` compila otimizado no perfil dev (sem isso, segundos por hash nos testes).
  - Testes de rota de `auth` viraram pasta (`tests/api/auth/`: Discord, senha, vínculo).
  - Front: `/[locale]/login` (Discord ou senha), `/[locale]/signup` e `/[locale]/account` (vincular e desvincular o Discord, criar, trocar e remover a senha). O nome no cabeçalho leva à conta. `account` e `signup` entraram nos slugs reservados.
- **Inscrição, visibilidade e link de inscrição** (parte do item 1 do duelo; decisões em "Torneios" no `decisions.md`):
  - Migration: `tournaments.visibility` (`public`/`private`), `participants` (id bigint, tenant, torneio, membro, convite usado, quem inscreveu) com FK para `tenant_members`, e `tenant_invites.tournament_id` (link de inscrição, sem validade).
  - API: domínio `participants/` (listar, inscrever-se e sair pelo botão, organizador inscrever e tirar; regras em `registration.rs`, que o aceite do convite reusa). `tournaments/visibility.rs` decide quem vê (`can_see`, `find_visible`); lista e página escondem rascunho e privado de quem não pode ver. `invites/tournament.rs` gera e lista os links de um torneio; prévia e aceite tratam os dois tipos, e aceitar o de torneio entra no tenant e inscreve numa transação só, com o torneio travado. O PATCH recusa vagas abaixo dos inscritos e muda a visibilidade até encerrar.
  - Front: visibilidade no formulário, inscrição na página do torneio (botão, inscritos, organizador inscrever e tirar, links de inscrição), `/invite/[token]` com a prévia do torneio, e a lista do tenant com privado, "inscrito" e ocupação.

## Padrões assumidos (configuráveis, confirmar com o usuário quando surgir)

- Melhor de N: um valor padrão + um para a decisiva (final na simples; grande final e reset na dupla). Ex.: md3 + md5.
- Grande final com reset: opção do organizador.
- Ban de campeão pelo capitão; pick por cada jogador. Timeout no veto: escolha aleatória, desfazível pelo organizador.
- Corrigir resultado = refazer o estado com `replay` e o histórico corrigido; correção que contradiz partida posterior é recusada.

## Próximo: duelo de ponta a ponta

Um incremento por vez, cada um com migration, rotas (OpenAPI), testes `sqlx::test`, tipos regenerados no front e tela mínima.

1. **Check-in e seeding** (a inscrição já existe)
   - `participants` ganha seed e check-in. Janela de check-in; quem não fizer sai antes da chave.
   - Data de início do torneio entra aqui, junto com a janela de check-in, e com ela as transições inscrição → check-in → em andamento.
   - Seeding manual, aleatório (seed salva para reproduzir) ou por rating (quando existir).
   - Decidir se o apelido é único dentro do tenant (em aberto no `decisions.md`).
   - `engine::core::ParticipantId` ainda é `Uuid`; com `participants` em bigint, trocar por um id inteiro (regra de ids do `decisions.md`).
2. **Chave e resultados**
   - Gerar a chave com `EliminationBracket`; persistir só config, participantes e `match_reports` (histórico); reconstruir por `replay`.
   - Ao gerar: rodar `TournamentSetup::validate` de novo com o número real de inscritos e o pool atual.
   - Envio de resultado por um lado, janela de contestação, organizador resolve contestação.
   - Front: visualização da chave (simples e dupla), página da partida.

## Depois

- Membros: remover membro, sair do tenant, transferir a posse.

- Tempo real (WebSocket no Axum) para chave, check-in e partida.
- Veto de mapas e pick/ban de campeões ao vivo, com regra de não repetição aplicada.
- Prints do placar (S3 genérico, URL assinada, WebP, apagar 30 dias após o fim).
- Rating (Elo, configurável por tenant).
- Outros formatos: FFA por pontos/frags, 2v2/3v3 com formação de times, round robin, suíço.
- Deploy: Render (API), Vercel (web), Supabase (Postgres), CI no GitHub Actions conferindo que o `.sqlx/` está em dia (`SQLX_OFFLINE=true cargo check`).
