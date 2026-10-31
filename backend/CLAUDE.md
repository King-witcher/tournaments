# Backend (backend/)

Organização do código. Siga ao criar arquivo, módulo, rota ou teste. Regras gerais (ids, tenant, erros, migrations) estão no `CLAUDE.md` da raiz.

## Crates

- `engine`: regras de torneio, puras (sem I/O, banco nem relógio). Organização em `crates/engine/CLAUDE.md`.
- `api`: HTTP e banco. Carrega dados, chama o `engine` e grava; não reimplementa regra de formato, pontuação, veto, campeões nem rating.
- `migrate`: só aplica as migrations.

## api: um módulo por domínio

```
crates/api/src/
  lib.rs, main.rs, config.rs   AppState/MIGRATOR/app() (junta o router() de cada domínio), subida, variáveis de ambiente
  shared/         erro, extractors, tokens, texto, slug, idioma, Redis, Storage (S3), client_ip, origin; não conhece domínio nenhum
  outbox/         eventos de mudança e avisos, gravados na transação de quem muda (emit)
  mail/           email sem domínio: endereço canônico, layout, envio (Mailer), limites
  auth/           login, sessão, CurrentUser, cadastro, recuperação de senha, troca de email, avisos de segurança
  tenants/        comunidades e a base de todo dado de tenant: TenantId, Role, TenantMember
  members/  invites/  tournaments/  participants/  teams/  lifecycle/
  matches/        chave, resultados mapa a mapa e o veto (veto/)
  my_tournaments/ organizing/ notifications/ ratings/ screenshots/ push/ discord/
  realtime/       WebSocket, despachante do outbox, agendador (acima de todos)
  health.rs  maps.rs   domínios pequenos, um arquivo cada
```

O que cada domínio faz, de quem depende e o papel de cada arquivo estão no `//!` do `mod.rs` dele: leia antes de mexer e atualize ao acrescentar arquivo ou responsabilidade. Fora do óbvio: `ratings` lê mapas e torneios direto no banco, sem chamar `matches`; `screenshots` fica acima de `matches`; `lifecycle` tem o sorteio dos times (lê o rating).

### Tempo real: mudou estado visível, `emit` na mesma transação

Toda função que muda algo que uma página mostra emite no outbox, na transação dela, antes do commit: `tournaments::changed(conn, id)` (dados, status, inscrições, check-in, seeding; `changed_with` também avisa quem saiu), `tenants::changed` (membros, papéis, apelidos, convites), `tenants::access_lost` (quem saiu); a chave emite sozinha pelo `settle` (passe a fase lida antes de gravar para `actions::respond`). Emita na função de domínio que faz a transição, não só no handler, para valer também quando outra função a chama (`lifecycle::leave_tournaments`). Escrita de uma linha que recebe `&PgPool` passa a receber `&mut PgConnection`, e o handler abre a transação. Aviso para pessoas é `outbox::Event::notice` com código e parâmetros, nunca texto. O que só vai ao canal do Discord (torneio iniciado, encerrado, série decidida) é `Event::Announce`; todo `Notice` e `Announce` vira mensagem no Discord se o código está no `CATALOG` de `discord/catalog.rs` (aviso novo: uma linha lá e a chave no `Discord.json` do front). O despachante acorda sozinho depois da requisição que emitiu; fora de requisição (socket, tarefa), `state.realtime.wake()`.

### O que é um domínio

- Um recurso da API, e a rota diz qual: `/tenants/{slug}/tournaments/...` é `tournaments`, não `tenants`.
- Domínio novo: pasta nova, `mod <domínio>;` e `.merge(<domínio>::router())` em `lib.rs`. Cabendo em ~150 linhas pode ser um arquivo; ao crescer, vira pasta com a forma abaixo.

### Arquivos de um domínio

| Arquivo | Tem | Não tem |
|---|---|---|
| `mod.rs` | `//!` do domínio, os `mod`, `pub fn router()`, `pub use` do que outros domínios usam | lógica |
| `handlers.rs` | handlers com `#[utoipa::path]`: extrair, validar, chamar `db`, montar a resposta; constantes de regra (limite, validade) | SQL |
| `types.rs` | DTOs (`Serialize`/`Deserialize`/`ToSchema`) e structs de caminho | lógica, SQL |
| `db.rs` | toda query do domínio, inclusive transações | HTTP, extractor |
| `kv.rs` | o que o domínio guarda no Redis: chaves (o comentário do arquivo diz cada uma), TTL e comandos | HTTP, SQL |
| `<conceito>.rs` | regra pura ou peça com nome próprio (`slug.rs`, `session.rs`, `access.rs`), com testes unitários no arquivo | — |

Crie o arquivo só quando houver o que pôr nele, mas com esses nomes. Submódulos são privados (`mod db;`); o que sai do domínio sai pelo `pub use` do `mod.rs`.

### Dependências

- `shared` ← `outbox` e `mail` ← `auth` ← `tenants` ← demais domínios ← `discord` ← `realtime`. Nunca ao contrário, nunca em ciclo.
- Um domínio usa outro só pelo que o `mod.rs` dele exporta. Query que mais de um domínio precisa fica no domínio dono do conceito e sai pelo `mod.rs` (`tenants::my_tenant`); só vai para `shared` o que não tem conceito de domínio.

### db.rs e kv.rs

- `db.rs` recebe `&PgPool` (ou `&mut PgConnection`, passo de transação maior) e `TenantId` para dado de tenant. Devolve tipo do domínio, nunca o record anônimo do `query!`; erro `sqlx::Error` quando só o banco pode falhar, `ApiError` quando aplica regra (conflito, limite) ou lê enum do banco (`Role::from_db`). Número de regra é constante no `handlers.rs` e entra como parâmetro (`create_with_owner(.., MAX_OWNED_TENANTS)`).
- `kv.rs` recebe `&Redis`; toda chave sai de `redis.key(..)` (prefixo do ambiente e de cada teste). Comandos por `redis.query`/`redis.query_pipe`, nunca pela conexão crua (eles reconectam); escrita que precisa ir junto (sessão + índice) vai num pipe `atomic()` (MULTI). Erro `RedisError` vira 500 pelo `ApiError`. Pense na cota: no máximo uma leitura por requisição, e só onde não tem jeito (a sessão).

### Tamanho

- Handler com corpo acima de ~30 linhas: leve passos para `db.rs` ou um `<conceito>.rs`.
- Arquivo acima de ~300 linhas, sem testes: divida por conceito. Testes unitários acima de ~300 linhas: `tests/mod.rs` (helpers) + um arquivo por assunto, como em `engine/src/formats/elimination/tests/`.

### Testes de rota

- Em `crates/api/tests/api/`, num binário só (`main.rs` declara os módulos; arquivo solto em `tests/` vira outro binário). Um arquivo por domínio, com o nome do módulo em `src/`; helpers comuns (app, usuário com sessão, `redis(&db)`, `get`/`post`, `create`/`invite`/`join`) em `common.rs`.
- Teste com banco: `#[tokio::test]` e, na primeira linha, `let (db, _lease) = crate::db::test_db().await;` (nunca `#[sqlx::test]`). O `test_db` empresta um de poucos bancos reaproveitados, já esvaziado (por `delete`, nunca `truncate`); `_lease` segura o banco até o fim. Uma suíte por vez na máquina (lock de arquivo). Motivos em `docs/decisions/stack.md`, "Testes".
- Antes de escrever um teste, veja se outro já cobre o caso: um teste novo exercita um caminho, regra, código de erro ou constraint que nenhum outro exercita.
- Filtrar: `cargo test -p api invites::`. Regras do banco, por SQL direto: `schema/` para o schema inteiro, `matches/schema.rs` para resultados e histórico.
- Tempo real: o app de teste não sobe despachante nem agendador. `state(db)` + `app_from(state)` dão acesso ao `Realtime`; o teste chama `realtime::dispatch_once` e `realtime::tick` (com o `now` que quiser) e lê o que uma ação emitiu no outbox (`realtime::outbox`).

## Consistência do banco

O banco recusa estado inválido mesmo que a API erre (`docs/decisions/database.md`). Toda migration nova segue:

- **`on delete` decidido caso a caso**: `cascade` do pai para o que só existe dentro dele (tenant → membros, convites, torneios; torneio → tudo); `set null (coluna)` para autoria; recusa quando apagar deixaria a história incoerente (inscrito e fase de torneio iniciado, owner, usuário que jogou). FK de recusa é `no action`, nunca `restrict`: `no action` é conferida no fim do comando, então a cascata do pai passa. Trigger de recusa reconhece a cascata pelo pai já apagado (`tournament_started` é falso sem torneio ou tenant).
- **FK composta com `tenant_id`** (ou `tournament_id`) em toda referência entre dados de tenant, com alvo `unique (tenant_id, id)`. Autor dentro do tenant aponta para `tenant_members (tenant_id, user_id)`.
- **`check`** em todo enum (`in (...)`), faixa (a mesma da constante da API), formato (slug, email, hash) e estado (colunas que andam juntas, `(a is null) = (b is null)`). JSON gravado: função `*_valid(jsonb)` imutável com a forma mínima, que devolve falso (nunca erro) para qualquer entrada.
- **`not null`** em tudo que não é opcional (o significado do nulo vai no comentário da coluna); **`unique`** para toda regra de "um por" (parcial, com `where`, quando vale para parte das linhas).
- Regra entre linhas ou tabelas: trigger `before` (vale na hora) ou `constraint trigger ... deferrable initially deferred` (vale no commit, como "pelo menos um owner"). Erro com `errcode` (`check_violation` para estado, `foreign_key_violation` para exclusão recusada) e `constraint = '<tabela>_<regra>'`.
- **Dados existentes**: a migration roda em produção. Corrija antes de criar a regra, de forma determinística (e diga como no comentário), ou use `not valid` + `validate`. Teste o SQL numa cópia do banco local antes de aplicar. **Migration aplicada não muda mais, nem em comentário** (o sqlx confere o checksum do arquivo inteiro).
- **Teste por SQL direto**: para cada regra, gravar o estado inválido e conferir código e nome (`schema::rejected`); para cada cascata, apagar o pai e contar que nada sobrou.
- Horário de uma ação comparado com o de outra (enviado → contestado, inscrito → check-in) não vira `check`: cada uma grava o `now()` da própria transação, que pode ter começado antes.
