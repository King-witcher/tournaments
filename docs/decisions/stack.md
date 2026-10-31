# Produto, stack e convenções

## Produto

- Site para organizar torneios de Quake Champions: duelo, FFA, FFA por pontos/frags, 2v2, 3v3.
- **Nome: Telefrag** (do usuário, 31/10): termo do Quake sem a marca "Quake", igual em pt e es, e não amarra a plataforma a um torneio (a RageCon é uma comunidade dentro dela). `telefrag.com.br` e `telefrag.gg` estavam livres em 31/10. O nome interno (banco, Redis, repositório, Render) continua `tournaments`.
- **Logo**: o banner do usuário vetorizado à mão em `docs/brand/` (branco, como o original) e em `web/src/shared/lib/logo.ts` (os mesmos caminhos). No cabeçalho, na cor do texto do tema e só o ícone abaixo do `md` (a barra de ações não deixa espaço). Ícones do site (favicon, tela inicial) com o ícone em branco no fundo `#161616`, que funciona em aba clara e escura.
- Português e espanhol (a comunidade joga muito com argentinos).
- Multi-tenant: qualquer usuário logado cria tenants, com limite por usuário. Jogadores entram num tenant só por link de convite e têm um perfil por tenant.
- Primeiro formato de ponta a ponta: **duelo**. Exercita chave, veto, pick/ban, não repetição, resultado e rating; os outros reaproveitam.

## Stack

| Camada | Escolha |
|---|---|
| Backend | Rust + Axum + sqlx (SQL puro, sem ORM) + utoipa (OpenAPI em `/openapi.json`, Scalar em `/docs`) |
| Frontend | Next.js (App Router), **só apresentação**, next-intl, Bun (pacotes e scripts), Biome |
| Tipos no front | gerados do OpenAPI com openapi-typescript |
| Banco | Postgres. Dev: cluster local. Prod: database com usuário próprio no projeto Supabase us-east-1 existente |
| Redis | Dado efêmero (ver "Redis"). Dev: local, database 1. Prod: Upstash grátis, database 0 |
| Email | Resend pela API HTTP (ver `mail.md`) |
| Tempo real | WebSocket no Axum, `tokio::broadcast` em memória (uma instância); `LISTEN/NOTIFY` se um dia houver mais de uma |
| Storage | API S3 genérica, provedor por variável de ambiente. Prod: AWS S3 (ver `screenshots.md`) |
| Observabilidade | Sentry no Rust e no Next |
| Hospedagem | API no Render (Virgínia, free + uptime monitor). Front na Vercel (funções em `iad1`, perto da API). Domínio próprio |

- **Rust**: interesse em aprender; o Scalar em `/docs` (no lugar do Swagger UI) permite acompanhar e testar os endpoints.
- **Bun só como gerenciador de pacotes e executor de scripts.** A Vercel detecta o `bun.lock`; as funções seguem no runtime Node (o do Bun na Vercel ainda é beta), e o `next` local também roda em Node.
- **Nada de ferramenta global além de Rust e Bun.** Sem pnpm e sem sqlx-cli: as migrations são arquivos à mão, aplicados por `cargo run -p migrate` (sem macros de query, então compila antes de a tabela existir) e pela API ao subir. O build sem banco (Render) usa o modo offline do sqlx: `bun scripts/sqlx-prepare.ts` faz o papel do `cargo sqlx prepare` (com `SQLX_OFFLINE_DIR`, as próprias macros gravam o `.sqlx/`), que vai para o repositório; o Render compila com `SQLX_OFFLINE=true`.
- **Next só apresentação**: renderiza no servidor as páginas públicas (preview do link no Discord), cuida das rotas por idioma e nunca acessa o banco.
- **Supabase é só host de Postgres**, via connection string. Nada de Supabase Auth, Realtime, Storage ou supabase-js, para evitar lock-in.
- **Render free**: 750 h/mês por workspace, o que cabe em **um** serviço sempre ligado.
- **Render → Supabase**: a conexão direta é só IPv6 e costuma falhar no Render; use o pooler em **modo sessão**, com o usuário próprio (`usuario.<project-ref>`). Se o pooler não aceitar outro database, um schema dedicado dentro de `postgres`.

## Redis

Upstash grátis: 256 MB e 500 mil comandos por mês (~16 mil por dia); passar da cota faz os comandos falharem até o mês virar.

- **Só dado efêmero, que some sozinho com TTL**: sessões e tentativas de senha, o cadastro esperando confirmação (conta que ninguém confirma não chega ao banco), links de recuperação e de troca de email, limites de envio de email e por IP, o vínculo de provedor autorizado por 10 minutos. Sessão no Postgres sobrevivia a deploy, mas a tabela só crescia; no Redis expira sozinha, tem índice por conta e tira as escritas de login do banco.
- **Cota**: a sessão é o único comando no caminho de toda requisição logada (um `HGET`). Referência: noite de torneio com 32 jogadores recarregando a chave a cada 30 s por 3 horas, ~4 chamadas por página, dá ~46 mil comandos. Se apertar: pago por uso (US$ 0,20 por 100 mil comandos) ou o fixo de 250 MB (US$ 10/mês). Pipe e MULTI juntam idas e voltas, mas cada comando deles conta.
- **Fora, por ora**:
  - cache de leitura: chave e torneio mudam com o tempo, parte da resposta depende de quem vê, e cada acerto também gasta cota. Só depois de medir;
  - locks: a concorrência já é `select ... for update` na transação, mais correto que um lock fora dela. O agendador do tempo real usa advisory lock do Postgres, que não gasta cota;
  - tempo real entre instâncias (continua `tokio::broadcast`) e rate limit por conta nas escritas (não pedido).
- **Sem Redis a API não sobe** (`REDIS_URL` obrigatória, `PING` ao subir) e, se cair, as rotas que leem a sessão respondem 500. O `/health` não consulta o Redis (o monitor de uptime gastaria a cota). A API confere o Redis antes das migrations: sem ele, o deploy falha sem mexer no banco e a instância antiga segue no ar.
- **Conexão**: `ConnectionManager` do redis-rs (abre no primeiro comando, reconecta; o comando que falha por conexão parada tenta de novo uma vez). TLS (`rediss://`) pelo rustls com o provedor aws-lc instalado no `main`: com ring (sqlx) e aws-lc (reqwest) ativos o rustls não escolhe sozinho e entra em pânico.
- **Database e prefixo**: um database só (local, o 1: os outros são de outros projetos do usuário; Upstash, o 0, o único). Toda chave passa por `Redis::key` com `REDIS_PREFIX` (padrão `tournaments:`), para checkout principal e worktrees dividirem o database. Os testes usam `test:{banco}:{empréstimo}:`; a execução seguinte, com o lock da suíte, apaga todo `test:*` antes do primeiro teste (senão chegaram a 39 mil chaves; hoje ~1.900).

## Ids

- **Chave interna**: `bigint generated always as identity` (users, tenants, convites e o que vier). O Postgres é um só e gera os ids, então UUID/ULID não ajudam: 8 bytes contra 16, sequenciais (índice compacto) e cabem no `number` do JS. `tenant_id` também é bigint (num índice composto com outro bigint, um `int4` não economiza por causa do alinhamento).
- **Não escondemos esses ids**: toda rota que recebe um id confere a participação no tenant, então enumerar não dá acesso a nada.
- **Tabela de ligação usa chave natural**: `tenant_members (tenant_id, user_id)`, `user_identities (provider, provider_user_id)`, `participants (tournament_id, user_id)`. Quem ocupa um lado da chave é o time (`teams`, id bigint), que vira o `ParticipantId` (`engine::core::ParticipantId(i64)`; o `engine` não depende de `uuid`).
- **Identificador público e legível**: slug (tenant, torneio). Usuário não tem identificador público; se um dia tiver perfil público, ganha um handle.
- **Segredos** (sessão, convite, links por email): aleatórios, só o SHA-256 guardado (convite no banco; sessão e links no Redis). Convite e links têm 128 bits em base58 (22 caracteres, sem `0/O/I/l`, para o duplo clique selecionar tudo); a sessão, 256 bits em base64url (só no cookie).
- **Exceção: o começo do token de convite**. `tenant_invites.token_hint` guarda os 4 primeiros caracteres em claro, para a organização reconhecer o link na lista, saber qual revogar e ver por qual link cada membro entrou (`joined_via`, só para organizer e owner) se um link vazar. Sobram ~105 bits desconhecidos; o resto continua hash.
- Se um dia houver ids gerados fora do banco: `uuidv7()`, nativo no PG18.

## Repositório e CI

- Privado agora, público depois: nenhum segredo no código (`.env.example` versionado), histórico limpo.
- GitHub Actions com cache de build Rust (o plano privado tem 2.000 min/mês). Em pull request roda só o lado cujos arquivos mudaram (`dorny/paths-filter`); em push na `main`, os dois; execução antiga do mesmo ref é cancelada.
- A conferência do `.sqlx/` regrava as queries num diretório à parte e compara (`diff -r`), reaproveitando as dependências compiladas (~1 min), em vez de rodar `sqlx-prepare.ts` (compilaria tudo de novo). Existe porque `.sqlx/` sobrando ou diferente seria invisível ao clippy offline.
- Banco de dev: cluster Postgres local; usuário `tournaments` sem senha (`trust` no `pg_hba.conf`), restrito ao próprio banco e aos de teste.

## Organização do código

Regras em `backend/CLAUDE.md` e `web/CLAUDE.md` (o Claude Code os carrega ao abrir arquivos da pasta; uma skill dependeria de ser acionada e um doc importado na raiz custaria contexto em toda sessão).

- **Por domínio dos dois lados**, com os mesmos nomes, e `shared/` para o genérico.
- **API: SQL só em `<domínio>/db.rs`**: achar uma query é abrir um arquivo, e handler com transação não fica longo.
- **Testes de rota num binário só** (`tests/api/`), um arquivo por domínio: no Windows cada binário de teste demora para linkar.
- **Front sem shadcn por enquanto**: primitivos próprios em `shared/ui`, com os nomes de variante do shadcn, para ele entrar depois sem mudar quem usa.

## Testes

Uma suíte levou perto de uma hora com outras sessões rodando; um cluster Postgres separado para os testes foi recusado.

- **Bancos reaproveitados** (`tests/api/db.rs`), no lugar do `sqlx::test` (criava e apagava um banco por teste; o `drop database` força um checkpoint no servidor inteiro, 107 s com três suítes): cada banco principal tem 8 bancos de teste (`TEST_DATABASES`), `_sqlx_test_<banco>_t<n>`, migrados uma vez e recriados só quando as migrations aplicadas não batem com as do código. Antes de cada teste as tabelas são esvaziadas e as linhas das migrations (campeões, mapas) voltam de uma cópia. Descartado: um banco só (os testes contam com banco vazio e rodam em paralelo) e uma transação por teste (os handlers fazem commit). A suíte foi de ~1 hora para ~40 s.
- **Esvaziar por `delete`, não `truncate`**: o `truncate ... restart identity` troca os arquivos de cada tabela, e no Windows com antivírus isso custava 0,13 s por teste parado e 9 s com outras sessões compilando (suíte de 65 s a 760 s). O `delete` de tabelas quase vazias fica na memória: 9 a 23 s. Comunidades e usuários saem primeiro (a cascata leva o resto) e cada tabela numa subtransação, repetindo a que um trigger recusou. `session_replication_role = replica` pediria `grant` de superusuário em cada máquina. 8 bancos porque 16 ganha pouco e, com o pool cheio (8 conexões por teste), passaria das 100 conexões do Postgres.
- **Uma suíte por vez na máquina**: o binário pega um lock de arquivo na pasta temporária (`tournaments-api-tests.lock`); a de outra sessão espera. Dez suítes juntas levaram o mesmo que em fila, mas ninguém via resultado antes do fim, e os testes do Discord que dependem do relógio falharam sob carga.
- **Teste que não cobre nada novo sai**: só fica o que exercita um caminho, regra, código de erro ou constraint que nenhum outro exercita. Juntar testes pequenos num só ficou de fora: um teste a mais custa milissegundos, e o separado diz melhor o que quebrou.
