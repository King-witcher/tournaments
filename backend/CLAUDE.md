# Backend (backend/)

Organização do código. Siga ao criar arquivo, módulo, rota ou teste. Regras gerais do projeto (ids, tenant, erros, migrations) estão no `CLAUDE.md` da raiz.

## Crates

- `engine`: regras de torneio, puras (sem I/O, banco nem relógio). Organização em `crates/engine/CLAUDE.md`.
- `api`: HTTP e banco. Carrega dados, chama o `engine` e grava. Não reimplementa regra de formato, pontuação, veto, campeões nem rating.
- `migrate`: só aplica as migrations.

## api: um módulo por domínio

```
crates/api/src/
  lib.rs          AppState, MIGRATOR e app(), que junta o router() de cada domínio
  main.rs         lê config, conecta, migra, sobe
  config.rs       variáveis de ambiente
  shared/         erro, extractors (o IP de quem pede: client_ip.rs), tokens e códigos, texto, slug, idioma, Redis,
                  Storage (o bucket S3 genérico), a recusa de escrita vinda de outra origem (origin.rs);
                  não conhece nenhum domínio
  outbox/         eventos de mudança e avisos, gravados na transação de quem muda (emit)
  mail/           email sem domínio: forma canônica do endereço, layout, envio (Mailer) e limites (por endereço,
                  por dia e, nos pedidos sem login, por IP)
  auth/           login (Discord, Google, username e senha), sessão, CurrentUser; cadastro (por código ou
                  link), recuperação de senha e troca de email por link; avisos por email do que muda no acesso
  tenants/        comunidades e a base de todo dado de tenant: TenantId, Role, TenantMember
  members/        membros: listar, perfil, papel, apelido, sair, remover
  invites/        convites da comunidade e links de inscrição em torneio
  tournaments/    torneio, traduções, fases, quem vê, agenda e check-in automático
  participants/   inscrição (individual, por user_id) e check-in
  teams/          times: quem ocupa um lado da chave em todos os modos (no duelo, o time de um
                  jogador, criado pelo banco com a inscrição), seeding e desistência do time;
                  no TDM, a formação, o nome, o capitão e a substituição
  lifecycle/      transições depois de publicar (check-in, reabrir, iniciar, desfazer) e desistência;
                  o sorteio dos times do TDM (lê o rating)
  matches/        chave e resultados, mapa a mapa, e o veto de mapas e campeões (veto/); com
                  screenshot obrigatório, o envio exige o screenshot (evidence.rs); slots.rs, a
                  partida para screenshots
  my_tournaments/ GET /me/tournaments: os torneios de quem vê, com a situação e a próxima partida
  organizing/     GET /me/organizing: o que espera a organização em todas as comunidades
  ratings/        ranking Elo por comunidade: cache refeito do zero quando os triggers marcam o tenant (agendador, leitura e as rotas daqui); lê mapas e torneios direto no banco, sem chamar matches
  screenshots/    screenshots do placar por mapa no bucket: PUT assinado conferido por HEAD, quem anexa
                  e remove, URL assinada para ver, rotina diária de exclusão. Fica acima de matches
  push/           Web Push: inscrições dos navegadores e envio
  discord/        avisos da comunidade no Discord por webhook: configuração do owner, fila e envio
  realtime/       WebSocket, despachante do outbox, agendador (com a rotina diária dos screenshots)
  health.rs       domínios pequenos, um arquivo cada
  maps.rs
```

O que cada domínio faz, de quem depende e o papel de cada arquivo estão no `//!` do `mod.rs` dele: leia antes de mexer, e atualize-o quando acrescentar um arquivo ou uma responsabilidade ao domínio.

O veto (mapas e campeões) fica em `matches/veto/`: depende da fase refeita e do `settle` da chave.

### Tempo real: mudou estado visível, `emit` na mesma transação

Toda função que muda algo que uma página mostra emite no outbox, na transação dela, antes do commit: `tournaments::changed(conn, id)` (dados, status, inscrições, check-in, seeding; `changed_with` também avisa quem saiu dos inscritos), `tenants::changed` (membros, papéis, apelidos, convites), `tenants::access_lost` (quem saiu), e a chave emite sozinha pelo `settle` (passe a fase lida antes de gravar para `actions::respond`). Emita na função de domínio que faz a transição, não só no handler, para valer também quando outra função a chama (`lifecycle::leave_tournaments`). Escrita de uma linha só que hoje recebe `&PgPool` passa a receber `&mut PgConnection`, e o handler abre a transação. Aviso para pessoas é `outbox::Event::notice` com código e parâmetros, nunca texto. O que só vai para o canal do Discord da comunidade (torneio iniciado, encerrado, série decidida) é `Event::Announce`; todo `Notice` e `Announce` também vira mensagem no Discord se o código está no `CATALOG` de `discord/catalog.rs` (aviso novo para pessoas: uma linha lá e a chave no `Discord.json` do front). O despachante acorda sozinho depois da requisição que emitiu; fora de requisição (socket, tarefa), acorde com `state.realtime.wake()`.

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
| `kv.rs` | o que o domínio guarda no Redis: formato das chaves (o comentário do arquivo diz cada uma), TTL e comandos | HTTP, SQL |
| `<conceito>.rs` | regra pura ou peça com nome próprio (`slug.rs`, `nickname.rs`, `session.rs`, `access.rs`, `discord.rs`), com os testes unitários no próprio arquivo | — |

Crie o arquivo só quando houver o que pôr nele, mas com esses nomes. Submódulos são privados (`mod db;`); o que sai do domínio sai pelo `pub use` do `mod.rs`.

### Dependências

- `shared` ← `outbox` e `mail` ← `auth` ← `tenants` ← demais domínios ← `discord` ← `realtime`. Nunca ao contrário, nunca em ciclo.
- Um domínio usa outro só pelo que o `mod.rs` dele exporta (`crate::tenants::{TenantMember, my_tenant}`).
- Query que mais de um domínio precisa fica no domínio dono do conceito e sai pelo `mod.rs` (ex.: `tenants::my_tenant`, usado por `invites`). Só vai para `shared` o que não tem conceito de domínio nenhum.

### db.rs

- Recebe `&PgPool` (ou `&mut PgConnection`, quando é um passo de transação maior) e `TenantId` para dado de tenant.
- Devolve tipo do domínio, nunca o record anônimo do `query!`. Erro `sqlx::Error` quando só o banco pode falhar; `ApiError` quando aplica regra (conflito, limite) ou lê enum do banco (`Role::from_db`).
- Número de regra (limite, validade) é constante no `handlers.rs` e entra como parâmetro (`create_with_owner(.., MAX_OWNED_TENANTS)`).

### kv.rs

- Recebe `&Redis`; toda chave sai de `redis.key(..)`, que põe o prefixo do ambiente (e o de cada teste).
- Comandos por `redis.query`/`redis.query_pipe` (`Cmd::hget(..)`, `redis::pipe()`), nunca pela conexão crua: eles reconectam. Escrita que precisa ir junto (sessão + índice) vai num pipe `atomic()` (MULTI).
- Devolve tipo do domínio; erro `RedisError`, que vira 500 pelo `ApiError`.
- Pense na cota (cada comando conta): uma leitura por requisição no máximo, e só onde não tem jeito (a sessão). Pipe junta idas e voltas, mas cada comando dele conta.

### Tamanho

- Handler com corpo acima de ~30 linhas: leve passos para `db.rs` ou para um `<conceito>.rs`.
- Arquivo acima de ~300 linhas, sem contar testes: divida por conceito.
- Testes unitários acima de ~300 linhas: `tests/mod.rs` (helpers) + um arquivo por assunto, como em `engine/src/formats/elimination/tests/`.

### Testes de rota

- Em `crates/api/tests/api/`, num binário só: `main.rs` declara os módulos. Arquivo solto em `tests/` vira outro binário, que linka a crate inteira de novo.
- Teste que usa o banco: `#[tokio::test]` e, na primeira linha, `let (db, _lease) = crate::db::test_db().await;` (nunca `#[sqlx::test]`, que cria e apaga um banco por teste e custava um checkpoint do Postgres a cada um). O `test_db` (`db.rs`) empresta um de poucos bancos (`TEST_DATABASES`, padrão 8), migrados uma vez e mantidos entre execuções, já esvaziado (por `delete`, nunca `truncate`, que no Windows custava segundos por teste; as sequências voltam ao 1 e campeões e mapas ao que as migrations gravam); o `_lease` segura o banco até o fim do teste. Uma suíte por vez na máquina (lock de arquivo): a de outra sessão espera, com aviso no stderr.
- Antes de escrever um teste, veja se outro já cobre o caso: um teste novo é um caminho, regra, código de erro ou constraint que nenhum outro exercita, não a mesma verificação com outros dados.
- Um arquivo por domínio, com o nome do módulo em `src/`. Helpers comuns (app, usuário com sessão, `redis(&db)` com o prefixo do teste, `get`/`post`/..., `create`/`invite`/`join`) em `common.rs`; helper de um domínio só fica no arquivo dele.
- Filtrar: `cargo test -p api invites::`.
- Regras do banco, por SQL direto: `schema/` para o schema inteiro (exclusões, owner, checks), `matches/schema.rs` para resultados e histórico.
- Tempo real: o app de teste não sobe o despachante nem o agendador. `state(db)` + `app_from(state)` dão acesso ao `Realtime`, e o teste chama `realtime::dispatch_once` e `realtime::tick` (com o `now` que quiser); o que uma ação emitiu se lê no outbox (`realtime::outbox` em `tests/api/realtime/`).

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
