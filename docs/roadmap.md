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
- **Check-in e seeding** (item 1 do duelo; decisões em "Torneios" no `decisions.md`):
  - Migration: `tournaments.starts_at`, `check_in_minutes` (padrão 30, nulo sem check-in), `registration_reopened_at` e `seeding_seed`; `participants.checked_in_at`, `checked_in_by` e `seed` (único por torneio).
  - `engine`: `ParticipantId(i64)` (saiu o `uuid`) e `seeding::random_order` (ChaCha8 do `chacha20` com a seed, Fisher–Yates, sorteio sem viés; teste fixa a saída).
  - API: `tournaments/schedule.rs` (data, janela, abertura automática do check-in aplicada por `find`/`list` ao ler, PATCH da data e da janela) e `tournaments/db/` dividido em `read`, `status`, `translations`; publicar exige data. `participants/check_in.rs` (o próprio e o organizador, só no check-in), `seeding.rs` (ver, sortear, ordenar; `finalize` ao iniciar) e `seed_order.rs` (elegíveis e ordem, puro). Domínio novo `lifecycle/` com abrir check-in, reabrir inscrição, iniciar e desfazer início, todas por `transition::apply`. `shared::nullable` para PATCH com `null`. Testes de rota de `tournaments` e `participants` viraram pasta (`schedule`, `check_in`, `seeding`) e há `lifecycle.rs`.
  - Front: data (no fuso de quem preenche) e janela no formulário; página com início no fuso de quem vê (`LocalDateTime`), aviso de quando o check-in abre, botão de check-in, marca de check-in e seed nos inscritos; para o organizador, marcar e desmarcar, `LifecyclePanel` (abrir check-in, reabrir, iniciar e desfazer, com confirmação) e painel de seeding (sortear, subir/descer, salvar). Feature nova `lifecycle`.

- **Chave e resultados** (fecha o duelo de ponta a ponta, sem veto; decisões em "Resultados" no `decisions.md`):
  - `engine`: `series::validate_games` (detalhe por mapa: um por mapa jogado, sem empate nem mapa repetido, vencedores batendo com o placar e o último mapa do vencedor), `MapId`, `Occupant` e `EliminationBracket::occupants`.
  - Migration: `maps` (16 mapas, pool de duelo provisório), `tournaments.contest_minutes` (padrão 5), `match_results` (resultado vigente por partida, com `position` no replay), `match_games` (detalhe por mapa) e `match_events` (histórico), com FKs compostas para o torneio e os participantes e `check` em cada invariante de série e de estado. Testes que gravam estado inválido direto por SQL.
  - API: `maps.rs` (`GET /maps`) e domínio `matches/`: `GET .../bracket` e `.../matches/{match}` (a chave sai sempre do replay dos confirmados), `POST .../report`, `.../confirm`, `.../contest` e `PUT .../result` (organizador: lança, resolve, corrige, W.O.). Iniciar roda `TournamentSetup::validate` com os elegíveis e o pool de agora; desfazer apaga resultados e histórico. Confirmação por tempo ao ler e agir (`due.rs`), encerramento e reabertura automáticos (`stage::sync_status`). Testes em `tests/api/matches/` (chave, envio, organizador, correções, banco).
  - Front: feature `matches` (chave simples e dupla com destaque das partidas de quem vê e do que esperam dele, contagem regressiva, classificação; página `/[tenant]/[torneio]/matches/[partida]` com envio, detalhe por mapa, confirmar e contestar, painel do organizador e histórico), `Badge` e `Countdown` em `shared/ui`, janela de contestação no formulário e desfazer início com contagem de resultados e o endereço digitado.

- **Consistência do banco** (decisões em "Consistência do banco" no `decisions.md`; checklist para migration nova no `backend/CLAUDE.md`):
  - Migration de exclusões e posse: usuário owner ou que jogou torneio iniciado não sai; sessões, logins e tentativas de senha saem com ele, a autoria fica nula; tenant e torneio apagam tudo o que têm, inclusive resultados; inscrito e fase de torneio iniciado só saem com ele (e não entram nem mudam seed e check-in); exatamente um owner por tenant (índice único + trigger no commit, para a transferência); autor de inscrição, check-in e convite é membro do mesmo tenant.
  - Migration de checks: data de início fora do rascunho, check-in só com janela, tradução fora do idioma original, ao menos uma fase, vagas acima dos inscritos, slugs e idioma original fixos, forma do JSON de campeões e fases, convite da inscrição do mesmo torneio, formato de username, hash, tokens e avatar, campeão com nome único. Dados antigos corrigidos antes (torneio publicado sem data ganhou a de criação, com a abertura automática suspensa).
  - Testes por SQL direto em `tests/api/schema/` (`deletion`, `owner`, `checks`), com um torneio encerrado com resultados, mapas, histórico, traduções e convites para conferir as cascatas.

- **Membros: remover, sair, transferir a posse e desistência** (decisões em "Multi-tenant" e "Resultados" no `decisions.md`):
  - Migration de membro inativo: `tenant_members.left_at` e `removed_by`; o owner nunca inativo; sair só com os convites revogados e sem inscrição em torneio que não começou; inativo não é inscrito nem abre convite; desfazer o início não deixa inscrito inativo (conferido no commit).
  - Migration de desistência: `participants.withdrawn_at` e `withdrawn_by` (só em andamento, só com seed, não se desfaz), `confirmed_via = withdrawal` e evento `auto_walkover` (sem placar e sem autor, perdedor desistente ou inativo), checks trocados com nome explícito.
  - API: `DELETE /tenants/{slug}/me` (sair) e `DELETE /tenants/{slug}/members/{user_id}` (owner remove qualquer um; organizer, players), em `members/departure.rs`, que fica acima de `lifecycle` e `invites`: tira das inscrições abertas, desiste dos em andamento e revoga os convites do membro. `TenantMember`, listas e inscrição só veem membro ativo; aceitar convite reativa a mesma linha como player. `POST .../participants/{user_id}/withdrawal` (organizador) em `lifecycle/withdrawal.rs`. `matches/withdrawals.rs` dá o W.O. automático depois de toda mudança (em `actions::respond`, na confirmação por tempo e na desistência), em loop para a dupla. Desfazer o início tira desistentes e inativos. `POST /tenants/{slug}/transfer-ownership` (slug digitado, limite de 3 do novo owner, antigo vira organizer). `StoredTournament` passou a levar o `tenant`.
  - Testes: `members/departure`, `lifecycle/withdrawal` (W.O. agora e depois na dupla, pendente substituído, dois lados fora, confirmação por tempo, desfazer), `tenants/transfer` e, por SQL direto, `schema/inactive` e `schema/withdrawal`.
  - Front: "Remover" na lista de membros para quem pode, "Sair da comunidade" e, para o owner, "Transferir a posse" (escolher o membro e digitar o endereço), com o que acontece em cada confirmação; "Desistência" para o organizador em cada inscrito da chave de torneio em andamento; marca "saiu"/"desistiu" nos inscritos, na chave, na partida e na classificação; textos e erros novos em pt e es.

## Padrões assumidos (configuráveis, confirmar com o usuário quando surgir)

- Melhor de N: um valor padrão + um para a decisiva (final na simples; grande final e reset na dupla). Ex.: md3 + md5.
- Grande final com reset: opção do organizador.
- Ban de campeão pelo capitão; pick por cada jogador. Timeout no veto: escolha aleatória, desfazível pelo organizador.
- Corrigir resultado = refazer o estado com `replay` e o histórico corrigido; correção que contradiz partida posterior é recusada.

## Próximo

O duelo já vai da inscrição ao campeão. Um incremento por vez, cada um com migration, rotas (OpenAPI), testes `sqlx::test`, tipos regenerados no front e tela mínima.

1. **Tempo real** (WebSocket no Axum) para chave, check-in e partida. Um agendador chama `matches::apply_due_confirmations` (hoje aplicada ao ler e agir; ela já dá os W.O. automáticos que a confirmação deixa prontos) para os torneios com pendente vencido e avisa quem vê. Eventos do item de membros a avisar: desistência (organizador ou saída da comunidade) e W.O. automático (mudam a chave e as partidas de um torneio em andamento; `lifecycle::leave_tournaments` pode mexer em vários torneios de uma vez), membro saiu ou foi removido (lista de membros; quem foi removido perde o acesso na hora) e posse transferida (papéis de dois membros).
2. **Veto** de mapas e pick/ban de campeões ao vivo, com regra de não repetição aplicada. Encaixa entre a partida ficar pronta e o envio do resultado: o pool por torneio sai de `maps`, e o detalhe por mapa do envio passa a ter de seguir os mapas do veto. Partida de quem desistiu nunca chega ao veto: vira W.O. ao ficar pronta.
3. **Prints do placar** (S3 genérico, URL assinada, WebP, apagar 30 dias após o fim).
4. **Rating** (Elo, configurável por tenant), por mapa a partir de `match_games` (sem detalhe, pelos mapas vencidos). Decidir se W.O. (do organizador e automático) conta no rating.

## Depois

- Outros formatos: FFA por pontos/frags, 2v2/3v3 com formação de times, round robin, suíço.
- Deploy: Render (API), Vercel (web), Supabase (Postgres), CI no GitHub Actions conferindo que o `.sqlx/` está em dia (`SQLX_OFFLINE=true cargo check`).
