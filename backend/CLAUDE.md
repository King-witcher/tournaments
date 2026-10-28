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
  tenants/        tenants (criar, ver, transferir a posse), e a base de todo dado de tenant:
                  TenantId, Role, TenantMember, apelido
  members/        /tenants/{slug}/me (apelido, sair) e /tenants/{slug}/members (listar,
                  papel, remover); departure.rs (quem remove quem; sair deixa o membro
                  inativo: sai das inscrições abertas e os convites dele são revogados).
                  Fica acima de lifecycle e invites
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
                  .../start): transition.rs concentra de onde, para onde e o que cada uma faz;
                  withdrawal.rs, a desistência (.../participants/{user_id}/withdrawal) e o membro
                  que sai da comunidade (sai das inscrições abertas, desiste das em andamento).
                  Fica acima de tournaments, participants e matches (iniciar fecha o seeding e
                  gera a chave; desfazer a descarta)
  maps.rs         domínio pequeno: GET /maps e a busca de ids por slug que matches usa
  my_tournaments/ GET /me/tournaments: os torneios de quem vê em todas as comunidades, com a
                  situação e a próxima partida (TenantMember::all_of, tournaments::list,
                  matches::place). Fica acima de tournaments e matches
  matches/        .../bracket, .../matches/{match}[/result] e .../matches/{match}/games/{n}
                  [/report|/confirm|/contest]: a série mapa a mapa. stage.rs (fase → chave por
                  replay das séries decididas, com os mapas; start, discard, sync_status),
                  games.rs (mapa gravado → engine), result.rs (série decidida), settle.rs (depois
                  de toda mudança: anula os mapas depois da decisão, fecha as séries, W.O.
                  automático, status), due.rs (confirmação por tempo dos mapas,
                  apply_due_confirmations), report.rs (jogadores), manage.rs (organizador:
                  lançar, resolver e corrigir mapas, W.O. na série), outcome.rs (mapa pelos lados
                  → engine), tasks.rs (o que a partida e cada mapa esperam de quem vê), key.rs
                  (wb-1-3), history.rs, view.rs, mine.rs (lugar de quem vê na chave e próxima partida), detail.rs, actions.rs, withdrawals.rs (W.O.
                  automático de quem desistiu; settle_withdrawals); SQL em db/ (leitura e
                  confirmação por tempo, writes da série, events, games)
```

Domínios previstos pelo roadmap: `veto` (pick/ban de mapas e campeões), `realtime` (WebSocket; o agendador chama `matches::apply_due_confirmations`), `media` (prints no S3), `ratings`.

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
- Regras do banco, por SQL direto: `schema/` para o schema inteiro (exclusões, owner, checks), `matches/schema.rs` para resultados e histórico.

## Consistência do banco

O banco recusa estado inválido mesmo que a API erre (decisões em "Consistência do banco" no `docs/decisions.md`). Toda migration nova segue:

- **`on delete` decidido caso a caso**, nunca o padrão por descuido:
  - `cascade` do pai para o que só existe dentro dele (tenant → membros, convites, torneios; torneio → tudo dele);
  - `set null (coluna)` para autoria (quem fez), com a coluna nula permitida;
  - recusa quando apagar deixaria a história incoerente (inscrito e fase de torneio iniciado, owner, usuário que jogou). FK de recusa é `no action`, nunca `restrict`: `no action` é conferida no fim do comando, então a cascata do pai passa. Trigger de recusa reconhece a cascata pelo pai já apagado (`tournament_started` é falso sem torneio ou sem tenant).
- **FK composta com `tenant_id`** (ou `tournament_id`) em toda referência entre dados de tenant, com alvo `unique (tenant_id, id)`. Autor dentro do tenant aponta para `tenant_members (tenant_id, user_id)`.
- **`check`** em todo enum (`in (...)`), faixa (a mesma da constante da API), formato (slug, username, tamanho do hash) e estado (colunas que andam juntas, `(a is null) = (b is null)`; status que exige coluna). JSON gravado: função `*_valid(jsonb)` imutável com a forma mínima, que devolve falso (nunca erro) para qualquer entrada.
- **`not null`** em tudo que não é opcional; o significado do nulo fica no comentário da coluna.
- **`unique`** para toda regra de "um por" (parcial, com `where`, quando vale só para parte das linhas).
- Regra entre linhas ou tabelas: trigger `before` (vale na hora) ou `constraint trigger ... deferrable initially deferred` (vale no commit, como "pelo menos um owner"). Erro com `errcode` (`check_violation` para estado, `foreign_key_violation` para exclusão recusada) e `constraint = '<tabela>_<regra>'`.
- **Dados existentes**: a migration roda em produção. Corrija antes de criar a regra, de forma determinística (e diga como no comentário), ou use `not valid` + `validate`. Teste o SQL numa cópia do banco local antes de aplicar. Migration aplicada não muda mais.
- **Teste por SQL direto**: para cada regra, gravar o estado inválido e conferir código e nome (`schema::rejected`); para cada cascata, apagar o pai e contar que nada sobrou.
- Horário de uma ação comparado com o de outra (enviado → contestado, inscrito → check-in) não vira `check`: cada uma grava o `now()` da própria transação, que pode ter começado antes da outra.

## engine

```
crates/engine/src/
  core.rs         tipos de mais de um formato: ParticipantId, MapId, MatchId, Match, Occupant,
                  Outcome, EngineError
  series/         melhor de N: mod.rs (tamanho e placar, validate_score), games.rs (a série
                  mapa a mapa: próximo mapa, troca, onde a série fecha e o que fica depois dela)
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
