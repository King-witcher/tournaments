# Decisões do projeto

Registro das decisões tomadas antes do início do código, com o motivo de cada uma. Atualize quando algo mudar.

## Produto

- Site web para organizar torneios de Quake Champions: duelo, FFA, FFA por pontos/frags, 2v2, 3v3.
- Idiomas: português e espanhol (a comunidade joga muito com argentinos). Tudo localizável.
- Multi-tenant: qualquer usuário logado cria tenants, com limite por usuário. Torneios pertencem a um tenant.
- Jogadores entram num tenant só por link de convite e têm um perfil por tenant.
- Primeiro formato de ponta a ponta: **duelo**. Exercita chave, veto, pick/ban, não repetição, resultado e rating; os outros reaproveitam.

## Stack

| Camada | Escolha |
|---|---|
| Backend | Rust + Axum + sqlx (SQL puro, sem ORM) + utoipa (OpenAPI, Swagger UI em `/docs`) |
| Frontend | Next.js (App Router), **só apresentação**, next-intl, Bun (pacotes e scripts), Biome (lint e formatação) |
| Tipos no front | gerados do OpenAPI com openapi-typescript |
| Banco | Postgres. Dev: cluster local. Prod: database com usuário próprio no projeto Supabase us-east-1 existente |
| Tempo real | WebSocket no Axum, `tokio::broadcast` em memória (uma instância). `LISTEN/NOTIFY` se um dia houver mais de uma |
| Storage | API S3 genérica, provedor por variável de ambiente (Cloudflare R2 sugerido) |
| Observabilidade | Sentry no Rust e no Next |
| Hospedagem | API no Render (Virgínia, free + uptime monitor). Front na Vercel (funções em `iad1`, perto da API). Domínio próprio |

Motivos e cuidados:

- **Rust**: interesse em aprender; o Swagger UI permite acompanhar os endpoints.
- **Bun só como gerenciador de pacotes e executor de scripts.** A Vercel detecta o `bun.lock` e instala com `bun install`. As funções continuam no runtime Node (o runtime Bun na Vercel ainda é beta), e localmente o `next` também roda em Node.
- **Nada de ferramenta global além de Rust e Bun.** Sem pnpm e sem sqlx-cli: as migrations são arquivos criados à mão, aplicados pelo binário `cargo run -p migrate` (que não usa macros de query, então compila mesmo antes da tabela existir) e também pela API ao subir. O build sem banco (Render) usa o modo offline do sqlx: `bun scripts/sqlx-prepare.ts` faz o papel do `cargo sqlx prepare` sem o sqlx-cli (com `SQLX_OFFLINE_DIR` definido, as próprias macros gravam o `.sqlx/`), o `.sqlx/` vai para o repositório e o Render compila com `SQLX_OFFLINE=true`.
- **Next só apresentação**: renderiza no servidor as páginas públicas (preview do link no Discord) e cuida das rotas por idioma. Nunca acessa o banco.
- **Supabase é só host de Postgres**, via connection string. Nada de Supabase Auth, Realtime, Storage ou supabase-js, para evitar lock-in.
- **Render free**: 750 h/mês por workspace, o que cabe em **um** serviço sempre ligado.
- **Conexão Render → Supabase**: a conexão direta é só IPv6 e costuma falhar no Render; use o pooler em **modo sessão**. Teste cedo com o usuário próprio (formato `usuario.<project-ref>`). Se o pooler não aceitar outro database, use um schema dedicado dentro do database `postgres`.

## Autenticação

- Própria, em Rust. Dois logins: Discord (OAuth2) e nome de usuário + senha.
- `users` + `user_identities (provider, provider_user_id, UNIQUE(provider, provider_user_id))`: cada forma de login é uma linha (`discord`, `password`; Steam no futuro é só mais uma). No máximo um login de cada provedor por usuário (`UNIQUE(user_id, provider)`).
- **Username e senha**: o username fica em `provider_user_id` como digitado e o hash argon2id em `password_hash` (só nas linhas `password`). Formato: 3 a 20 letras ou números ASCII, `_`, `.` e `-` (só ASCII para não haver nomes que parecem iguais). Único sem diferenciar maiúsculas (índice em `lower(provider_user_id)`), e o login também não diferencia. No cadastro o username vira o nome exibido; trocar o username depois não muda o nome. Senha de 8 a 128 caracteres, sem regra de composição (NIST 800-63B).
- **Limite de tentativas por username, no banco** (`password_failures`): 5 senhas erradas em 15 minutos bloqueiam aquele username até o erro mais antigo sair da janela (429 `auth.too_many_attempts` com os minutos). Vale também para username inexistente, que confere contra um hash fixo para não revelar pelo tempo quem existe. Não usa IP (não depende do `X-Forwarded-For` do Render); o custo é que alguém pode travar o login por senha de outra pessoa por 15 minutos, e o Discord continua entrando.
- **Sem recuperação de conta por enquanto**: não há email. O Discord é o backup, e a tela de conta e o cadastro avisam isso. Recuperação por email fica para quando houver provedor de email.
- **Vincular e desvincular**: quem entrou pelo Discord pode criar username e senha, e quem tem senha pode vincular o Discord (pelo mesmo OAuth, com a intenção "link" no cookie do state). A conta nunca fica sem login: remover a senha exige Discord vinculado, e desvincular o Discord exige senha. Trocar ou remover a senha e desvincular o Discord pedem a senha atual (com o mesmo limite de tentativas); trocar a senha encerra as outras sessões. Desvincular o Discord tira o avatar, que vinha dele.
- Vincular um Discord que já é login de outra conta é recusado (`auth.discord_taken`); não há mesclagem de contas.
- Sessão em cookie `HttpOnly; Secure; SameSite=Lax; Domain=.<domínio>`, com as sessões numa tabela no Postgres.

## Multi-tenant

- Um banco só, com coluna `tenant_id` em tudo o que pertence a um tenant.
- Toda função de acesso a dados de tenant recebe um `TenantId` tipado (extractor do Axum). Esquecer o filtro vira erro de compilação. RLS pode entrar depois como segunda camada.
- Tenant na URL por caminho: `/{locale}/{tenant}/...`.
- Papéis em `tenant_members`: owner, organizer, player.
- Na interface, tenant se chama **comunidade** (es: comunidad).
- Limite: cada usuário é owner de no máximo **3** tenants, fixo no código (`MAX_OWNED_TENANTS`). Ser organizer ou player em outros não conta.
- Slug: o front sugere a partir do nome, o usuário ajusta na criação e depois ele **não muda** (links de convite e de torneio nunca quebram). Formato a-z, 0-9 e hífen simples, de 3 a 32 caracteres, com uma lista de palavras reservadas para rotas do front (`new`, `invite`...).
- Um owner por tenant: quem criou. Transferir a posse fica para depois.
- Convite: link **multiuso**, vale 7 dias, revogável. Owner e organizer geram; quem entra vira **player**. Só o hash do token fica no banco, então o link aparece uma vez, na criação. O link vai sem idioma (`/invite/{token}`) e o proxy do next-intl manda cada pessoa para o seu.
- Aceitar um convite sendo já membro não muda papel nem apelido.
- Gestão de membros: o owner alterna player ↔ organizer. Remover membro e sair do tenant ficam para depois.
- Página do tenant **só para membros**. Para quem é de fora, a API responde 404 `tenant.not_found`, o mesmo de um tenant inexistente, para não revelar que ele existe. A prévia do convite (`GET /invites/{token}`) é pública: ter o token já é o convite.
- Perfil por tenant: por enquanto só o **apelido** (`tenant_members.nickname`). Começa com o nome exibido do usuário (o do Discord ou o username do cadastro) e o próprio membro edita. Não é único dentro do tenant.
- Configurações do tenant: ranking ligado/desligado, sistema (Elo no MVP, Glicko-2 depois), parâmetros (K, rating inicial), rating único ou por modalidade.

## Ids

- **Chave interna**: `bigint generated always as identity` (users, tenants, convites e o que vier). O Postgres é um só e gera os ids, então UUID/ULID não trazem vantagem. São 8 bytes contra 16, sequenciais (índice compacto, sem a fragmentação do UUID aleatório) e cabem no `number` do JS. `tenant_id` também é bigint: num índice composto com outro bigint, um `int4` não economiza nada por causa do alinhamento.
- **Não escondemos esses ids.** Toda rota que recebe um id confere a participação no tenant, então enumerar não dá acesso a nada; no máximo revela a ordem e o total aproximado de cadastros.
- **Tabela de ligação usa chave natural**: `tenant_members (tenant_id, user_id)`, `user_identities (provider, provider_user_id)`.
- **Identificador público e legível**: slug (tenant; torneio por tenant depois). Usuário não tem identificador público; se um dia tiver perfil público, ganha um handle.
- **Segredos** (sessão, convite): aleatórios, e só o SHA-256 vai para o banco. O token de convite tem 128 bits em base58 (22 caracteres, sem `0/O/I/l` e sem pontuação, então um duplo clique seleciona o token inteiro). O de sessão tem 256 bits em base64url (vive só no cookie).
- Se um dia houver ids gerados fora do banco ou bancos que se fundem: `uuidv7()`, nativo no PG18 (a ideia do ULID, com tipo nativo).

## Torneios

- Endereço `/{locale}/{tenant}/{torneio}`. O slug segue as regras do tenant (sugerido a partir do nome, ajustável na criação, a-z, 0-9 e hífen, de 3 a 32), é único dentro do tenant e **não muda**. Há palavras reservadas para as rotas do tenant (`new`, `members`, `settings`...).
- **Só membros** veem torneios. Quem é de fora recebe 404 `tournament.not_found`. Rascunho só aparece para organizer e owner; para player também é 404.
- Organizer e owner criam e editam. Criar gera um rascunho; publicar abre a inscrição; só rascunho pode ser excluído.
- **Vagas obrigatórias** (2 a 1024). A checagem de campeões na criação usa o pior caso para esse número (o pior caso não diminui com mais participantes; há teste disso no `engine`) e roda de novo ao gerar a chave com os inscritos reais. As vagas também fecham a inscrição quando lotam.
- **Pool de campeões**: tabela global `champions` com os 17 atuais, e o pool do torneio é o conjunto ativo. Excluir campeões por torneio fica para depois.
- Bans de campeão por mapa: de 0 a 3, padrão 1.
- Não repetição na checagem: escopo partida usa o maior melhor de N; fase, o pior caso da fase mais longa; torneio, a soma dos piores casos das fases. Sem a regra, basta sobrar um campeão depois dos bans.
- Formato padrão no formulário de duelo: eliminação dupla com reset, md3 nas partidas e md5 na grande final.
- Edição: nome, descrição, regras e traduções até o torneio encerrar. Formato, vagas e regra de campeões até ele começar; a checagem roda de novo a cada mudança e ao publicar. Quando houver inscrição, vagas abaixo do número de inscritos serão recusadas.
- O idioma original não muda depois da criação. Tradução: uma por idioma diferente do original, com nome obrigatório; descrição ou regras vazias mostram as do original. A API devolve o original e as traduções, e o front escolhe pelo idioma de quem vê.
- Descrição e regras em **Markdown**, sem HTML cru (`react-markdown` no front).
- Erro com números sai como `{"error": "champions.pool_exhausted", "params": {"required": 32, "available": 17}}`, e o front interpola os números na tradução.

## Rating

- **Derivado, nunca editado direto**: reprocessar as partidas do tenant + ajustes manuais do organizador (com motivo e autor). Corrigir um resultado antigo → recalcular.
- Trait `RatingSystem` na crate `engine`. Elo primeiro.
- Rating único ou por modalidade (duelo, FFA, times), conforme a configuração do tenant.

## i18n

- A API devolve **códigos** (`match.already_reported`), nunca texto. Status, formatos e fases são enums no código, traduzidos no front.
- Datas em UTC no banco, exibidas no fuso de quem vê.
- Conteúdo do organizador: idioma original + traduções **opcionais** em `tournament_translations (tournament_id, locale, ...)`. Sem tradução, mostra o original.

## Formatos

- Dois eixos independentes:
  - **chaveamento**: eliminação simples, dupla, round robin, suíço, lobbies FFA, série de pontos;
  - **partida**: melhor de N mapas; no FFA, pontos por colocação **ou** soma de frags (configurável).
- Duelo, 2v2 e 3v3 são o mesmo motor com `team_size` diferente.
- O torneio tem **fases** (`stages`) no modelo desde o início; a interface pode começar com uma.
- Fluxo: inscrição individual → check-in (janela antes de começar; quem não fizer sai) → formação de times → seeding → chave.
- Qualquer membro do tenant se inscreve, inclusive organizer e owner: os papéis são cumulativos (organizer também é jogador), então a rota usa `TenantMember<Player>`.
- Times são por torneio, montados pelo organizador: sorteio aleatório, sorteio balanceado por rating (zigue-zague) ou manual (arrastar). O capitão é definido pelo organizador ou pelo sorteio.
- Seeding: manual, aleatório ou por rating.
- Toda aleatoriedade (sorteios, timeouts) recebe uma seed ou RNG por parâmetro e a seed é salva, para que o resultado seja reproduzível e auditável.

## Veto e campeões

- Veto de mapas ao vivo, em turnos, com cronômetro. O capitão faz o veto.
- Ban de campeão pelo capitão; pick de campeão por cada jogador.
- Timeout: ban ou pick aleatório; o organizador pode desfazer.
- Pools de mapas e campeões são tabelas, não constantes. Campeão novo no jogo = linha nova.
- Regra de não repetição **opcional**, configurável em dois eixos: quem (jogador | time) × escopo (partida | fase | torneio).
- Validação na criação: pior caso de jogos por participante comparado com o pool menos os bans. Se faltar campeão, bloqueia com mensagem com números ("no pior caso são necessários 24 campeões; o pool tem 17").

## Resultados

- Um lado envia o placar; os demais têm X minutos para contestar. Sem contestação, confirma. Contestação vai para o organizador. Vale para duelo, times e FFA.
- Print do placar: o torneio decide se é obrigatório.
- Upload direto do navegador para o bucket, com URL assinada gerada pela API. O front converte para WebP antes.
- Prints apagados 30 dias após o fim do torneio (rotina diária).

## Notificações

- MVP: só no site, em tempo real pelo WebSocket. Webhook e DM do Discord ficam para depois.

## Repositório

- Privado agora, público depois: nenhum segredo no código (`.env.example` versionado), histórico limpo desde o primeiro commit.
- GitHub Actions com cache de build Rust (o plano privado tem 2.000 min/mês).
- Banco de dev: cluster Postgres local (sqlx confere queries na compilação; `sqlx::test` cria um banco por teste). Usuário `tournaments` sem senha (`trust` no `pg_hba.conf`), restrito ao próprio banco e aos bancos de teste.

## Organização do código

Regras completas em `backend/CLAUDE.md` e `web/CLAUDE.md`, que o Claude Code carrega sozinho ao abrir arquivos dessas pastas. Uma skill dependeria de ser acionada e um doc importado na raiz custaria contexto em toda sessão.

- **Por domínio dos dois lados**, com os mesmos nomes (auth, tenants, members, invites, depois tournaments, matches...), e `shared/` para o genérico. A rota diz o domínio. Uma pasta `components/` com tudo junto ficaria difícil de navegar com o crescimento.
- **API: SQL só em `<domínio>/db.rs`**; `handlers.rs` faz HTTP, `types.rs` tem os DTOs. Achar uma query é abrir um arquivo, e handler com transação (criar torneio, reportar resultado) não fica longo.
- **Testes de rota num binário só** (`tests/api/`), um arquivo por domínio: no Windows cada binário de teste demora para linkar.
- **Front sem shadcn por enquanto**: primitivos próprios em `src/shared/ui` (Button, Input, Field, Alert, Avatar, Page), com os nomes de variante do shadcn, para ele entrar no mesmo lugar depois sem mudar quem usa.
- Componente ou arquivo que cresce demais ou acumula funções é quebrado; os sinais estão nos dois `CLAUDE.md`.

## Em aberto

- Apelido único dentro do tenant? Hoje pode repetir; decidir junto com a inscrição em torneios.
- Organizer inscrito no próprio torneio: pode resolver contestação ou corrigir resultado de partida em que joga? Decidir junto com chave e resultados.
- Duelo: sequência do veto (ex.: ban-ban-pick-pick-decider) e tempo do cronômetro.
- Janela de contestação padrão (X minutos) e duração da janela de check-in.
- Critérios de desempate (FFA por pontos e grupos).
- Rating para FFA e times (Elo par a par? média do time?).
- Substitutos em times.
