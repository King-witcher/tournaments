# Backend (backend/)

Organização do código. Siga ao criar arquivo, módulo, rota ou teste. Regras gerais do projeto (ids, tenant, erros, migrations) estão no `CLAUDE.md` da raiz.

## Crates

- `engine`: regras de torneio, puras (sem I/O, banco nem relógio).
- `api`: HTTP e banco. Carrega dados, chama o `engine` e grava. Não reimplementa regra de formato, pontuação, veto, campeões nem rating.
- `migrate`: só aplica as migrations.

## api: um módulo por domínio

```
crates/api/src/
  lib.rs          AppState, MIGRATOR e app(), que junta o router() de cada domínio
  main.rs         lê config, conecta, migra, sobe
  config.rs       variáveis de ambiente
  health.rs       domínio pequeno: um arquivo só
  shared/         o que todo domínio usa; não conhece nenhum domínio
    error.rs        ApiError (código + params numéricos), ErrorBody
    engine.rs       From<EngineError> for ApiError: um código por variante do engine
    extract.rs      JsonBody, PathParams, nullable (campo de PATCH em que null limpa)
    token.rs        random_token, random_secret, hash_token, constant_time_eq
    text.rs         clean_text (uma linha), clean_long_text (Markdown)
    slug.rs         formato de slug; reservados e códigos ficam em cada domínio
    locale.rs       Locale (pt, es)
  auth/           login com Discord e com username e senha (argon2, limite de tentativas),
                  ligar e desligar cada um, sessão em cookie, CurrentUser
  tenants/        tenants, e a base de todo dado de tenant: TenantId, Role, TenantMember, apelido
  members/        /tenants/{slug}/me e /tenants/{slug}/members
  invites/        /tenants/{slug}/invites, /invites/{token} e os links de inscrição
                  (/tenants/{slug}/tournaments/{t}/invites, em tournament.rs)
  tournaments/    /tenants/{slug}/tournaments/...: torneio, traduções, fases; TournamentId,
                  status, setup (vagas, campeões, fases → checagem do engine), edit (PATCH),
                  visibility (quem vê: can_see, find_visible), schedule (data de início,
                  janela de check-in e a abertura automática, aplicada em db/read.rs ao ler)
  participants/   /tenants/{slug}/tournaments/{t}/participants, .../registration, .../check-in
                  e .../seeding; registration.rs (os três caminhos de inscrição; o convite usa
                  register), check_in.rs, seeding.rs (finalize, que o início usa), seed_order.rs
  lifecycle/      transições depois de publicar (.../check-in/open, .../registration/reopen,
                  .../start): transition.rs concentra de onde, para onde e o que cada uma faz.
                  Fica acima de tournaments, participants e matches (iniciar fecha o seeding e
                  gera a chave; desfazer a descarta)
  maps.rs         domínio pequeno: GET /maps e a busca de ids por slug que matches usa
  matches/        .../bracket e .../matches/{match}[/report|/confirm|/contest|/result]:
                  stage.rs (fase → chave por replay, start, discard, sync_status), due.rs
                  (confirmação por tempo, apply_due_confirmations), report.rs (jogadores),
                  manage.rs (organizador: lançar, resolver, corrigir, W.O.), outcome.rs (placar
                  pelos lados → engine), key.rs (wb-1-3), result.rs, history.rs, view.rs,
                  detail.rs, actions.rs; SQL em db/ (leitura e confirmação por tempo, writes,
                  events, games)
```

Domínios previstos pelo roadmap: `veto` (pick/ban de mapas e campeões), `realtime` (WebSocket; o agendador chama `matches::due::apply_due_confirmations`), `media` (prints no S3), `ratings`.

### O que é um domínio

- É um recurso da API, e a rota diz qual: `/tenants/{slug}/tournaments/...` é `tournaments`, não `tenants`. Rota aninhada em `/tenants/{slug}` fica no domínio do recurso.
- Domínio novo: pasta nova, `mod <domínio>;` e `.merge(<domínio>::router())` em `lib.rs`.
- Domínio que cabe em ~150 linhas pode ser um arquivo (`health.rs`). Quando crescer ou precisar de um segundo arquivo, vira pasta com a forma abaixo.

### Arquivos de um domínio

| Arquivo | Tem | Não tem |
|---|---|---|
| `mod.rs` | `//!` do domínio, os `mod`, `pub fn router()`, `pub use` do que outros domínios usam | lógica |
| `handlers.rs` | handlers com `#[utoipa::path]`: extrair, validar, chamar `db`, montar a resposta. Constantes de regra (limite, validade) | SQL |
| `types.rs` | DTOs da API (`Serialize`/`Deserialize`/`ToSchema`) e structs de caminho (`MemberPath`) | lógica, SQL |
| `db.rs` | toda query do domínio (`query!`/`query_as!`), inclusive transações | HTTP, extractor |
| `<conceito>.rs` | regra pura ou peça com nome próprio (`slug.rs`, `nickname.rs`, `session.rs`, `access.rs`, `discord.rs`), com os testes unitários no próprio arquivo | — |

Crie o arquivo só quando houver o que pôr nele, mas com esses nomes. Submódulos são privados (`mod db;`); o que sai do domínio sai pelo `pub use` do `mod.rs`.

### Dependências

- `shared` ← `auth` ← `tenants` ← demais domínios. Nunca ao contrário, nunca em ciclo.
- Um domínio usa outro só pelo que o `mod.rs` dele exporta (`crate::tenants::{TenantMember, my_tenant}`).
- Query que mais de um domínio precisa fica no domínio dono do conceito e sai pelo `mod.rs` (ex.: `tenants::my_tenant`, usado por `invites`). Só vai para `shared` o que não tem conceito de domínio nenhum.

### db.rs

- Recebe `&PgPool` (ou `&mut PgConnection`, quando é um passo de transação maior) e `TenantId` para dado de tenant.
- Devolve tipo do domínio, nunca o record anônimo do `query!`. Erro `sqlx::Error` quando só o banco pode falhar; `ApiError` quando aplica regra (conflito, limite) ou lê enum do banco (`Role::from_db`).
- Número de regra (limite, validade) é constante no `handlers.rs` e entra como parâmetro (`create_with_owner(.., MAX_OWNED_TENANTS)`).

### Tamanho

- Handler com corpo acima de ~30 linhas: leve passos para `db.rs` ou para um `<conceito>.rs`.
- Arquivo acima de ~300 linhas, sem contar testes: divida por conceito.
- Testes unitários acima de ~300 linhas: `tests/mod.rs` (helpers) + um arquivo por assunto, como em `engine/src/formats/elimination/tests/`.

### Testes de rota

- Em `crates/api/tests/api/`, num binário só: `main.rs` declara os módulos. Arquivo solto em `tests/` vira outro binário, que linka a crate inteira de novo.
- Um arquivo por domínio, com o nome do módulo em `src/`. Helpers comuns (app, usuário com sessão, `get`/`post`/..., `create`/`invite`/`join`) em `common.rs`; helper de um domínio só fica no arquivo dele.
- Filtrar: `cargo test -p api invites::`.

## engine

```
crates/engine/src/
  core.rs         tipos de mais de um formato: ParticipantId, MapId, MatchId, Match, Occupant,
                  Outcome, EngineError
  series.rs       melhor de N: placar (validate_score) e detalhe por mapa (validate_games)
  seeding.rs      sorteio reproduzível pela seed (random_order, ChaCha8), tamanho e ordem da chave
  champions.rs    ChampionRules e o orçamento da regra de não repetição
  tournament.rs   TournamentSetup::validate: vagas, fases e campeões antes de gravar
  formats/
    mod.rs        StageFormat (o JSON gravado em tournament_stages.config)
    elimination/  mod.rs (config, estado, replay), build.rs (topologia), resolve.rs (estados), tests/
```

- O JSON de `StageFormat` e `ChampionRules` fica gravado no banco: mudar o serde deles exige migration dos dados. Os testes em `formats/tests.rs` fixam o formato.

- Um formato por pasta em `formats/` (`round_robin/`, `swiss/`, `ffa/`): `mod.rs` com a API pública (config, estado, `replay`) e um arquivo por etapa.
- O que não é formato fica no topo, como `scoring/` (pontos e desempate de FFA), `veto/` e `rating/`. Um arquivo até ~300 linhas, depois pasta.
- Tipo usado por mais de um formato vai para `core.rs`.
