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
- **Nada de ferramenta global além de Rust e Bun.** Sem pnpm e sem sqlx-cli: as migrations são arquivos criados à mão, aplicados pelo binário `cargo run -p migrate` (que não usa macros de query, então compila mesmo antes da tabela existir) e também pela API ao subir. Quando o build sem banco precisar do `cargo sqlx prepare` (modo offline, no deploy), ele roda na CI ou instalado localmente no projeto.
- **Next só apresentação**: renderiza no servidor as páginas públicas (preview do link no Discord) e cuida das rotas por idioma. Nunca acessa o banco.
- **Supabase é só host de Postgres**, via connection string. Nada de Supabase Auth, Realtime, Storage ou supabase-js, para evitar lock-in.
- **Render free**: 750 h/mês por workspace, o que cabe em **um** serviço sempre ligado.
- **Conexão Render → Supabase**: a conexão direta é só IPv6 e costuma falhar no Render; use o pooler em **modo sessão**. Teste cedo com o usuário próprio (formato `usuario.<project-ref>`). Se o pooler não aceitar outro database, use um schema dedicado dentro do database `postgres`.

## Autenticação

- Própria, em Rust. MVP: login com Discord (OAuth2).
- `users` + `user_identities (provider, provider_user_id, UNIQUE(provider, provider_user_id))`: cada usuário pode ter N formas de login. Senha e Steam no futuro são só novas linhas.
- Login com senha, quando vier, exige argon2, limite de tentativas e recuperação de conta.
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
- Perfil por tenant: por enquanto só o **apelido** (`tenant_members.nickname`). Começa com o nome do Discord e o próprio membro edita. Não é único dentro do tenant.
- Configurações do tenant: ranking ligado/desligado, sistema (Elo no MVP, Glicko-2 depois), parâmetros (K, rating inicial), rating único ou por modalidade.

## Ids

- **Chave interna**: `bigint generated always as identity` (users, tenants, convites e o que vier). O Postgres é um só e gera os ids, então UUID/ULID não trazem vantagem. São 8 bytes contra 16, sequenciais (índice compacto, sem a fragmentação do UUID aleatório) e cabem no `number` do JS. `tenant_id` também é bigint: num índice composto com outro bigint, um `int4` não economiza nada por causa do alinhamento.
- **Não escondemos esses ids.** Toda rota que recebe um id confere a participação no tenant, então enumerar não dá acesso a nada; no máximo revela a ordem e o total aproximado de cadastros.
- **Tabela de ligação usa chave natural**: `tenant_members (tenant_id, user_id)`, `user_identities (provider, provider_user_id)`.
- **Identificador público e legível**: slug (tenant; torneio por tenant depois). Usuário não tem identificador público; se um dia tiver perfil público, ganha um handle.
- **Segredos** (sessão, convite): aleatórios, e só o SHA-256 vai para o banco. O token de convite tem 128 bits em base58 (22 caracteres, sem `0/O/I/l` e sem pontuação, então um duplo clique seleciona o token inteiro). O de sessão tem 256 bits em base64url (vive só no cookie).
- Se um dia houver ids gerados fora do banco ou bancos que se fundem: `uuidv7()`, nativo no PG18 (a ideia do ULID, com tipo nativo).

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

## Em aberto

- Apelido único dentro do tenant? Hoje pode repetir; decidir junto com a inscrição em torneios.
- Duelo: tipo de chave padrão (simples/dupla), melhor de N por fase, sequência do veto (ex.: ban-ban-pick-pick-decider), tempo do cronômetro.
- Janela de contestação padrão (X minutos) e duração da janela de check-in.
- Critérios de desempate (FFA por pontos e grupos).
- Rating para FFA e times (Elo par a par? média do time?).
- Substitutos em times.
