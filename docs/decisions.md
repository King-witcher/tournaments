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
| Backend | Rust + Axum + sqlx (SQL puro, sem ORM) + utoipa (OpenAPI em `/openapi.json`, Scalar em `/docs`) |
| Frontend | Next.js (App Router), **só apresentação**, next-intl, Bun (pacotes e scripts), Biome (lint e formatação) |
| Tipos no front | gerados do OpenAPI com openapi-typescript |
| Banco | Postgres. Dev: cluster local. Prod: database com usuário próprio no projeto Supabase us-east-1 existente |
| Tempo real | WebSocket no Axum, `tokio::broadcast` em memória (uma instância). `LISTEN/NOTIFY` se um dia houver mais de uma |
| Storage | API S3 genérica, provedor por variável de ambiente (Cloudflare R2 sugerido) |
| Observabilidade | Sentry no Rust e no Next |
| Hospedagem | API no Render (Virgínia, free + uptime monitor). Front na Vercel (funções em `iad1`, perto da API). Domínio próprio |

Motivos e cuidados:

- **Rust**: interesse em aprender; o Scalar em `/docs` permite acompanhar e testar os endpoints (trocou o Swagger UI, mais elegante).
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
- Um owner por tenant: quem criou, até transferir a posse. O banco garante exatamente um (ver "Consistência do banco").
- Convite: link **multiuso**, vale 7 dias, revogável. Owner e organizer geram; quem entra vira **player**. Só o hash do token fica no banco, então o link aparece uma vez, na criação. O link vai sem idioma (`/invite/{token}`) e o proxy do next-intl manda cada pessoa para o seu. O link de inscrição de um torneio (ver "Torneios") também faz entrar no tenant.
- Aceitar um convite sendo já membro não muda papel nem apelido.
- Gestão de membros: o owner alterna player ↔ organizer, remove membros e transfere a posse; o organizer remove players; cada um sai sozinho.
- **Sair ou ser removido deixa o membro inativo, com o histórico** (`tenant_members.left_at`, e `removed_by` quando alguém removeu): a linha não é apagada (o banco nem deixaria, se ele jogou torneio iniciado). O inativo perde o acesso (a API responde como para quem é de fora: 404 `tenant.not_found`), some da lista de membros e de "Minhas comunidades" e não pode ser inscrito. Sai das inscrições dos torneios que não começaram (rascunho, inscrição, check-in), desiste dos em andamento em que está na chave (W.O. nas partidas pendentes, ver "Resultados") e, nos encerrados, fica como está. Os convites que ele criou são revogados (a FK composta só os apagaria com a linha). Nas chaves e listas de torneios iniciados o nome continua, com a marca "saiu".
- **Quem remove**: o owner remove qualquer um (menos ele mesmo); o organizer remove só players (organizer → 403 `tenant.forbidden`). Ninguém remove o owner (409 `member.is_owner`), e remover a si mesmo é 409 `member.remove_self` (sair é outra rota). Qualquer membro sai sozinho, menos o owner, que transfere a posse antes (409 `member.owner_must_transfer`). Rotas: `DELETE /tenants/{slug}/members/{user_id}` e `DELETE /tenants/{slug}/me`.
- **Remover não é banimento** (assumido): quem foi removido volta por qualquer convite válido. Para impedir, revogue os convites.
- **Voltar por convite reativa a mesma linha**: apelido e histórico de antes, **como player** (assumido, também para quem era organizer); `joined_at` continua o da primeira entrada e `invite_id` passa a ser o convite da volta (assumido). Vale para o link de inscrição de torneio também.
- **Desfazer o início** de um torneio tira as inscrições de quem saiu da comunidade ou desistiu (assumido): fora de andamento, inscrito inativo ou desistente não fica.
- **Transferir a posse** (`POST /tenants/{slug}/transfer-ownership` com `user_id` e `confirm_slug`): o owner escolhe um membro ativo (player ou organizer) e digita o slug da comunidade, que a API confere (422 `tenant.confirmation_mismatch`). O novo owner respeita o limite de 3 tenants como owner (409 `member.owner_limit_reached` `{max}`, contado com a linha do usuário travada, como na criação). O antigo owner vira **organizer**. Numa transação, rebaixa o owner atual antes de promover o novo (a ordem inversa bate no índice único).
- Página do tenant **só para membros**. Para quem é de fora, a API responde 404 `tenant.not_found`, o mesmo de um tenant inexistente, para não revelar que ele existe. A prévia do convite (`GET /invites/{token}`) é pública: ter o token já é o convite.
- Perfil por tenant: por enquanto só o **apelido** (`tenant_members.nickname`). Começa com o nome exibido do usuário (o do Discord ou o username do cadastro) e o próprio membro edita. **Pode repetir** dentro do tenant, sem tratamento (decidido junto com o check-in): as telas mostram o avatar ao lado, e as rotas identificam o membro pelo `user_id`.
- Configurações do tenant: ranking ligado/desligado, sistema (Elo no MVP, Glicko-2 depois), parâmetros (K, rating inicial), rating único ou por modalidade.

## Ids

- **Chave interna**: `bigint generated always as identity` (users, tenants, convites e o que vier). O Postgres é um só e gera os ids, então UUID/ULID não trazem vantagem. São 8 bytes contra 16, sequenciais (índice compacto, sem a fragmentação do UUID aleatório) e cabem no `number` do JS. `tenant_id` também é bigint: num índice composto com outro bigint, um `int4` não economiza nada por causa do alinhamento.
- **Não escondemos esses ids.** Toda rota que recebe um id confere a participação no tenant, então enumerar não dá acesso a nada; no máximo revela a ordem e o total aproximado de cadastros.
- **Tabela de ligação usa chave natural**: `tenant_members (tenant_id, user_id)`, `user_identities (provider, provider_user_id)`. Exceção: `participants` tem id próprio (bigint) além do `unique (tournament_id, user_id)`, porque ele vira o `ParticipantId` da chave (`engine::core::ParticipantId(i64)`; o `engine` não depende mais de `uuid`); nas rotas, o inscrito é identificado pelo `user_id`.
- **Identificador público e legível**: slug (tenant; torneio por tenant depois). Usuário não tem identificador público; se um dia tiver perfil público, ganha um handle.
- **Segredos** (sessão, convite): aleatórios, e só o SHA-256 vai para o banco. O token de convite tem 128 bits em base58 (22 caracteres, sem `0/O/I/l` e sem pontuação, então um duplo clique seleciona o token inteiro). O de sessão tem 256 bits em base64url (vive só no cookie).
- Se um dia houver ids gerados fora do banco ou bancos que se fundem: `uuidv7()`, nativo no PG18 (a ideia do ULID, com tipo nativo).

## Torneios

- Endereço `/{locale}/{tenant}/{torneio}`. O slug segue as regras do tenant (sugerido a partir do nome, ajustável na criação, a-z, 0-9 e hífen, de 3 a 32), é único dentro do tenant e **não muda**. Há palavras reservadas para as rotas do tenant (`new`, `members`, `settings`...).
- **Só membros** veem torneios. Quem é de fora recebe 404 `tenant.not_found`. Rascunho só aparece para organizer e owner; para player é 404 `tournament.not_found`.
- **Visibilidade**: público (padrão) ou privado. O público aparece para todos os membros; o privado, só para os inscritos, organizers e owner, e para os outros membros some da lista e responde 404 `tournament.not_found`, como se não existisse. Muda até o torneio encerrar.
- Organizer e owner criam e editam. Criar gera um rascunho; publicar abre a inscrição; só rascunho pode ser excluído.
- **Vagas obrigatórias** (2 a 1024). A checagem de campeões na criação usa o pior caso para esse número (o pior caso não diminui com mais participantes; há teste disso no `engine`) e roda de novo ao gerar a chave com os inscritos reais. As vagas também fecham a inscrição quando lotam.
- **Pool de campeões**: tabela global `champions` com os 17 atuais, e o pool do torneio é o conjunto ativo. Excluir campeões por torneio fica para depois.
- Bans de campeão por mapa: de 0 a 3, padrão 1.
- Não repetição na checagem: escopo partida usa o maior melhor de N; fase, o pior caso da fase mais longa; torneio, a soma dos piores casos das fases. Sem a regra, basta sobrar um campeão depois dos bans.
- Formato padrão no formulário de duelo: eliminação dupla com reset, md3 nas partidas e md5 na grande final.
- Edição: nome, descrição, regras, traduções e visibilidade até o torneio encerrar. Formato, vagas, regra de campeões, data de início e janela de check-in até ele começar; a checagem roda de novo a cada mudança e ao publicar. Vagas abaixo do número de inscritos são recusadas (`tournament.max_participants_below_registered`).
- **Data e hora de início** (`starts_at`, UTC). O rascunho pode ficar sem; publicar exige (`tournament.starts_at_required`), e depois de publicado ela não sai mais. O organizador digita no fuso dele (o navegador converte para UTC) e cada um vê no próprio fuso (formatado no navegador; no HTML do servidor sai em UTC até hidratar). Data no passado não é recusada.
- **Check-in**: janela por torneio, em minutos antes do início (padrão 30; a API aceita de 5 a 1440 e o formulário oferece 15, 30, 45 e 60), ou **sem check-in**. Sem check-in não existe o status `check_in`: a inscrição vai direto para em andamento.
- **Ciclo depois de publicar**: inscrição → check-in → em andamento. O check-in **abre sozinho** quando chega `starts_at - janela`, avaliado pela API ao ler o torneio (lista ou página), sem job em segundo plano: a primeira leitura depois da hora grava a transição (com guarda contra mudança concorrente). O relógio fica na API, nunca no `engine`. Só anda para a frente: adiar o início com o check-in aberto não o fecha.
- O **início é sempre manual**: organizer ou owner clica em "Iniciar" a partir do check-in (ou da inscrição, sem check-in). O organizador também pode **abrir o check-in antes da hora**, **reabrir a inscrição** durante o check-in (os check-ins feitos continuam) e **desfazer o início** (volta para o check-in, ou para a inscrição sem check-in), para quando iniciou por engano. Desligar o check-in durante o check-in volta para a inscrição.
- **Reabrir a inscrição suspende a abertura automática** da janela que já tinha aberto: a API guarda quando reabriu (`registration_reopened_at`) e só reabre sozinha uma janela cuja abertura seja posterior a esse momento, ou seja, quando a data de início ou a janela mudam para depois. Assim a ordem entre reabrir e mudar a data não importa, e mudar a data só um pouco (para uma abertura que já tinha passado) não fecha a inscrição de novo na hora. A tela avisa que a automática está suspensa.
- **Check-in** se faz e desfaz só durante o check-in: o próprio inscrito, e organizer ou owner para qualquer inscrito (fica gravado quem marcou). Quem não fez continua inscrito (histórico), marcado sem check-in, mas fica fora do seeding e da chave. Inscrição manual pelo organizador durante o check-in entra **sem** check-in. Depois de iniciar, inscrições e check-ins congelam.
- **Seeding** na revisão antes de iniciar, só para organizer e owner: os elegíveis (com check-in; sem check-in no torneio, todos os inscritos) na ordem que vale. "Sortear" gera uma seed u64 aleatória na API, grava no torneio e embaralha pelo `engine` (`seeding::random_order`, ChaCha8 com a seed: a mesma seed e os mesmos elegíveis dão a mesma ordem em qualquer versão; um teste fixa o resultado). O ajuste manual manda a ordem completa de `user_id`s; ela precisa ter exatamente os elegíveis (`seeding.order_invalid` se alguém fez ou desfez check-in no meio). Seeds ficam em `participants.seed`.
- **Ao iniciar**: vale a ordem gravada; elegíveis sem seed vão para o fim, na ordem de inscrição; se ninguém tem seed, sorteia (e grava a seed). Os seeds são compactados para 1..N entre os elegíveis e os outros ficam sem. Mínimo de 2 elegíveis (`tournament.not_enough_eligible`). Seeding por rating fica para quando houver rating.
- **Inscrição**, por três caminhos, sempre de membros do tenant:
  - **botão**: o próprio membro, de qualquer papel (ver "Formatos"), só em torneio **público** e com a inscrição aberta; no privado, organizer e owner se inscrevem pela inserção manual;
  - **organizador**: organizer ou owner inscreve qualquer membro, em público ou privado, do rascunho até o torneio começar;
  - **link de inscrição**: o que um torneio privado usa para trazer gente, e também serve no público.
- Lotou, fecha (`tournament.full`). O inscrito sai sozinho enquanto a inscrição está aberta; depois disso, só o organizador tira, até o torneio começar. Inscrever de novo quem já está inscrito não dá erro.
- **Link de inscrição**: é um convite do tenant que também inscreve. Mesma tabela (`tenant_invites.tournament_id`), mesmo endereço (`/invite/{token}`), multiuso, revogável pela mesma rota. Não tem data: vale enquanto a inscrição estiver aberta e pode ser gerado já no rascunho, para mandar antes de publicar. Aceitar entra no tenant como player (quem já é membro mantém papel e apelido) e inscreve **numa transação só**: sem vaga ou fora da inscrição, não entra em nada.
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
- **Trocar o idioma**: seletor no cabeçalho, que mantém a página; cada idioma aparece no próprio nome ("Español" também para quem lê em português).
- **Espanhol**: a chave do torneio é sempre **llave** (llave de ganadores, llave de perdedores), mais comum na Argentina que "cuadro" (assumido; antes as duas palavras se misturavam).

## Formatos

- Dois eixos independentes:
  - **chaveamento**: eliminação simples, dupla, round robin, suíço, lobbies FFA, série de pontos;
  - **partida**: melhor de N mapas; no FFA, pontos por colocação **ou** soma de frags (configurável).
- Duelo, 2v2 e 3v3 são o mesmo motor com `team_size` diferente.
- O torneio tem **fases** (`stages`) no modelo desde o início; a interface pode começar com uma.
- Fluxo: inscrição individual → check-in (janela antes de começar; quem não fizer fica fora da chave) → formação de times → seeding → chave. Detalhes do check-in e do seeding em "Torneios".
- Qualquer membro do tenant se inscreve, inclusive organizer e owner: os papéis são cumulativos (organizer também é jogador), então a rota usa `TenantMember<Player>`.
- Times são por torneio, montados pelo organizador: sorteio aleatório, sorteio balanceado por rating (zigue-zague) ou manual (arrastar). O capitão é definido pelo organizador ou pelo sorteio.
- Seeding: manual, aleatório (seed salva) ou por rating (quando houver rating).
- Toda aleatoriedade (sorteios, timeouts) recebe uma seed ou RNG por parâmetro e a seed é salva, para que o resultado seja reproduzível e auditável.

## Veto e campeões

- Veto de mapas ao vivo, em turnos, com cronômetro. O capitão faz o veto.
- Ban de campeão pelo capitão; pick de campeão por cada jogador.
- Timeout: ban ou pick aleatório; o organizador pode desfazer.
- Pools de mapas e campeões são tabelas, não constantes. Campeão novo no jogo = linha nova.
- Regra de não repetição **opcional**, configurável em dois eixos: quem (jogador | time) × escopo (partida | fase | torneio).
- Validação na criação: pior caso de jogos por participante comparado com o pool menos os bans. Se faltar campeão, bloqueia com mensagem com números ("no pior caso são necessários 24 campeões; o pool tem 17").

## Resultados

- Um lado envia o resultado; os demais têm uma janela para contestar. Sem contestação, confirma. Contestação vai para o organizador. Vale para duelo, times e FFA. No duelo (e nos times), **cada mapa da série é enviado e confirmado sozinho** (pedido do usuário em 28/10: a série inteira de uma vez era ruim de usar).
- **A chave nasce no "Iniciar"**, na mesma transação: `TournamentSetup::validate` roda de novo com o número real de elegíveis e o pool de campeões de agora (`champions.pool_exhausted` com os números se faltar), e a chave é montada com os elegíveis na ordem dos seeds. **Nada do estado do `engine` é gravado**: ficam a configuração (`tournament_stages`), os participantes (seeds 1..N) e os resultados (`match_results`); a chave é sempre refeita por `replay` com os confirmados na ordem em que passaram a valer (`position`, mantida nas correções), e cada resultado também guarda os lados com que entrou, conferidos no replay.
- **Uma fase por enquanto**: os resultados já têm `stage`; uma fase seguinte teria como participantes a classificação da anterior, montada quando ela termina (`matches/stage.rs`).
- **Identificador de partida** nas rotas e no banco: o `MatchId` legível, `wb-1-3` (chave de cima; na simples, tudo é `wb`), `lb-2-1` (de baixo), `gf-1-1` e `gf-2-1` (grande final e reset). Rotas `.../bracket` e `.../matches/{partida}`; chave e partidas seguem a visibilidade do torneio.
- **Envio, mapa a mapa**: qualquer um dos dois jogadores de uma partida pronta envia o mapa N (`POST .../matches/{partida}/games/{n}/report`): quem venceu (pelo lado), o mapa jogado (um mapa ativo, obrigatório para o jogador) e os frags de cada lado (opcionais, os dois ou nenhum, de 0 a 999, sem empate e com mais frags para quem venceu). O número vai no caminho, para dois envios simultâneos do mesmo mapa não virarem dois mapas (`match.game_already_reported`). Quem não joga não envia (`match.not_player`); o organizador que não joga lança.
- **Regras da série** (do `engine`, `series::games`; a API só carrega, chama e grava):
  - o mapa N+1 exige o N **enviado**, não confirmado (`match.game_out_of_order` `{next}`); quem enviou um mapa já pode mandar o próximo;
  - o mapa não se repete na série (`match.map_repeated`), até o veto existir;
  - **não há mapa depois da decisão**: o próximo é recusado quando os mapas enviados e não contestados já dão a maioria a alguém (`match.series_decided`, inclusive com eles ainda pendentes) ou quando a série já teve os N mapas (`match.no_games_left`);
  - **mapa contestado não conta** para essa decisão (o vencedor dele está em dúvida), então contestar o mapa N não trava o N+1, nem quando o contestado decidiria a série: se os jogadores jogaram o mapa seguinte, é porque o resultado contestado provavelmente estava errado;
  - quando a contestação se resolve (ou o organizador corrige um mapa), os mapas que passam a ficar depois da decisão são **anulados** (apagados, com o evento `voided` no histórico). Assumido: o organizador decidiu o mapa, então o seguinte não existiria; o rules-reviewer sugeriu segurar o próximo mapa quando o contestado decidiria a série, recusado porque travaria o caso comum (o adversário contesta porque ele venceu e os dois seguem jogando);
  - **a série fecha sozinha** quando os mapas confirmados, em sequência desde o primeiro, dão a maioria (2 na md3, 3 na md5): todos os mapas até o que deu a maioria precisam estar confirmados (um contestado no meio segura a série, porque o placar ainda pode mudar). Só então a série entra em `match_results` (`confirmed_via = maps`, com o placar dos mapas e o horário da confirmação que faltava) e a chave avança pelo `replay`, como antes; o evento `decided` registra o placar.
- **Resposta, por mapa**: o adversário confirma ou contesta cada mapa com um comentário (1 a 280 caracteres); contestado sai da janela e espera o organizador. Quem enviou não confirma nem contesta o próprio mapa.
- **Janela de contestação**: por torneio, padrão **10 minutos** (era 5; os torneios que ainda não tinham começado e estavam com 5, o padrão antigo, passaram para 10 na migration `20261101000000_contest_window`; os já iniciados ficaram como estavam), a API aceita de 1 a 1440 e o formulário oferece 2, 5, 10, 15, 30 e 60. Vale **por mapa**: muda até o torneio encerrar e vale para os envios seguintes (o prazo fica gravado em cada mapa). Sem resposta, o mapa confirma sozinho no fim da janela, avaliado pela API ao ler a chave ou a partida e antes de cada ação (como a abertura do check-in, sem job; só trava o torneio quando há prazo vencido). `matches::due::apply_due_confirmations` é a peça isolada que o agendador do tempo real vai chamar.
- **A chave só avança com a série decidida** (pelos mapas confirmados, por W.O. do organizador ou pelo W.O. automático). Antes disso, a próxima partida continua esperando; a chave mostra o placar dos mapas confirmados e quando há mapa esperando confirmação ou contestado.
- **Organizador** (organizer ou owner, **inclusive em partida que ele joga**; tudo fica no histórico com autor e horário): **confirma na hora** um mapa pendente (quando o adversário demora), resolve a contestação de um mapa (aceitando como foi enviado, ou com outro resultado), **lança** o próximo mapa e **corrige** qualquer mapa (`PUT .../games/{n}`, já confirmado), também com o torneio encerrado, e dá **W.O. na série inteira** (`PUT .../result`; só ele dá). No W.O., os mapas confirmados ficam (foram jogados) e os pendentes e contestados são anulados. A correção de um mapa refaz a série: se a série decidida muda de vencedor ou placar, a chave é refeita por `replay` com a série nova na mesma posição; se fica sem decisão (ex.: 2 x 0 vira 1 x 1), a série **reabre** (evento `reopened`) e a chave volta a esperar. A correção é recusada (`match.correction_conflicts`) se uma partida decidida depois deixa de fechar (vencedor que não chega lá, outros lados, reset que deixa de existir), e os mapas de partidas ainda abertas cujos lados mudaram são anulados (evento `voided`). Mapas que os mapas confirmados decidem por cima de um W.O. (o organizador lança o que faltava) substituem o W.O.
- **Fim**: quando a decisiva é confirmada (final da simples; grande final sem reset necessário, ou o reset), o torneio vira `finished` sozinho e mostra a classificação (`Placement`). Uma correção que volta a deixar partida por jogar (ex.: o reset passa a ser necessário) devolve o torneio para `in_progress`.
- **Desfazer o início** é permitido com resultados e apaga séries, mapas e histórico; a chave informa quantas partidas têm mapa ou resultado e a tela pede, havendo algum, que se digite o endereço do torneio. Só em andamento: depois de encerrado não se desfaz (assumido; a correção continua valendo).
- **Histórico** por partida (`match_events`): os eventos de um mapa levam o número dele (`game`) e o vencedor do mapa (enviado, confirmado, confirmado por tempo, contestado com o comentário, resolvido, lançado, corrigido, anulado); os da série, o vencedor e o placar (série decidida, reaberta, W.O., W.O. automático). Com autor, horário e nota opcional do organizador. Os eventos de antes do envio mapa a mapa ficam como eram (da série inteira). Só desfazer o início apaga.
- **Desistência**: num torneio em andamento, o inscrito da chave desiste quando o organizador marca (`POST .../participants/{user_id}/withdrawal`, botão "Desistência" na página do torneio) ou quando sai (ou é removido) da comunidade. Fica em `participants.withdrawn_at` (e `withdrawn_by`, o organizador ou quem removeu). Ele continua na chave e na lista, com a marca "desistiu" (ou "saiu"), e na comunidade, se foi só desistência. **Não se desfaz** (assumido; o organizador ainda corrige resultados). Marcar de novo não dá erro; inscrito fora da chave (sem seed) não desiste (409 `participant.not_in_bracket`). As confirmações por tempo vencidas valem antes.
- **W.O. automático**: toda partida pronta com um lado fora (desistiu ou saiu da comunidade) vira W.O. para o adversário, confirmado "pela desistência" (`confirmed_via = withdrawal`), sem autor, com o evento `auto_walkover`. É aplicado depois de toda mudança que pode deixar partida pronta (confirmação pelo adversário, pelo organizador ou por tempo, resolução, lançamento, correção, W.O. e a própria desistência), repetindo até não sobrar nenhuma: na dupla, quem perde por W.O. na chave de cima cai na de baixo e leva outro W.O. quando aquela partida ficar pronta. Assim nenhuma partida pronta fica esperando quem saiu, e ler a chave não precisa aplicar nada.
- **Mapa pendente ou contestado** na partida pronta de quem desiste é anulado, e a série vira W.O. para o adversário; os mapas confirmados ficam (assumido: quem desistiu não jogaria o resto de qualquer forma).
- **Os dois lados fora**: W.O. para o seed melhor (assumido); na dupla, ele cai na partida seguinte e a regra vale de novo.
- Quem sai da comunidade com o torneio já encerrado não é marcado desistente, mas conta como fora se uma correção reabrir o torneio.
- **Mapas**: tabela global `maps` (como `champions`), com `duel_pool`. Lista **provisória, para o usuário revisar**: pool de duelo = Awoken, Blood Covenant, Blood Run, Corrupted Keep, Deep Embrace, Exile, Insomnia, Molten Falls, Ruins of Sarnath, Vale of Pnath; fora do pool = Burial Chamber, Church of Azathoth, Lockbox, Tempest Shrine, The Longest Yard, Tower of Koth. Nome de mapa é nome próprio (não se traduz). `GET /maps` é pública. Até o veto definir um pool por torneio, o envio aceita qualquer mapa ativo, sem repetir na série (o formulário mostra o pool de duelo primeiro).
- **Rating por mapa** (item futuro): por isso cada mapa é uma linha de `match_games`, com o vencedor, o mapa e os frags.
- **Conversão das séries enviadas inteiras** (migration `20261101010000_games`, determinística): cada série com placar virou um mapa por mapa jogado. Com detalhe, cada mapa veio do detalhe (o vencedor é quem fez mais frags). **Sem detalhe, os mapas ficaram sem nome nem frags** (`map_id` nulo, permitido só para eles: o jogador sempre informa o mapa; o organizador pode manter sem ao corrigir), primeiro os do perdedor e depois os do vencedor, para o último ser de quem venceu. Cada mapa herdou o estado e a autoria do envio da série (quem enviou e quando, prazo, contestação com o comentário em todos os mapas da série contestada, confirmação). A série pendente ou contestada saiu de `match_results`: fecha quando os mapas confirmarem (pelo prazo que já tinham, ou pelo organizador). W.O. não tem mapas.
- **Banco rigoroso**: as invariantes também estão no banco. `match_games`: vencedor entre os lados, frags com mapa, sem empate e batendo com o vencedor, mapa sem repetir na série, envio de jogador com lado e prazo, contestado com comentário, confirmado com via e horário; no commit, os mapas de uma partida têm os mesmos lados e números de 1 a n (`match_games_sequence`). `match_results` só guarda a série decidida (`match_results_decided`); com ela, todos os mapas da partida estão confirmados e com os lados dela (`match_games_series_decided`) e o placar dela, fora o W.O., é o dos mapas (`match_results_games_score`). As FKs são compostas pelo torneio (participante do mesmo torneio, torneio do mesmo tenant) e participante ou fase de torneio iniciado não sai avulso (ver "Consistência do banco").
- Print do placar: o torneio decide se é obrigatório.
- Upload direto do navegador para o bucket, com URL assinada gerada pela API. O front converte para WebP antes.
- Prints apagados 30 dias após o fim do torneio (rotina diária).

## Consistência do banco

- O banco recusa estado inválido e entidade órfã mesmo que a API erre; a API continua conferindo antes, para devolver o código certo. Regras nas migrations `20261030000000_delete_rules` e `20261030010000_consistency_checks`, testes em `tests/api/schema/`, checklist para migration nova no `backend/CLAUDE.md`.
- **Excluir usuário** é recusado se ele for owner de algum tenant ou tiver participado de torneio iniciado (em andamento ou encerrado), para a chave antiga não perder um lado. Sessões, logins e tentativas de senha vão junto. A autoria (quem inscreveu, marcou o check-in, enviou, contestou ou confirmou um resultado, agiu no histórico) fica nula. Os convites que ele criou saem com a participação dele no tenant: convite é uma autorização dada pelo membro.
- **"Excluir conta"** (quando existir) vai **anonimizar** o usuário (nome, avatar, logins, sessões) em vez de apagar a linha, para os torneios que ele jogou continuarem inteiros.
- **Excluir tenant** apaga tudo o que está dentro. **Excluir torneio** também, inclusive resultados, mapas e histórico (a API só exclui rascunho, mas o banco aceita qualquer um).
- **Inscrito e fase de torneio iniciado** não saem avulsos, só com o torneio ou o tenant; também ninguém entra e seed e check-in não mudam. Por isso o membro inscrito num torneio iniciado também não sai do tenant: sair mantendo o histórico é ficar inativo, não linha apagada.
- **Membro inativo** (migration `20261031000000_inactive_members`): o owner nunca fica inativo; `removed_by` só em quem saiu, nunca ele mesmo, e fica nulo se o autor for apagado; sair exige os convites dele revogados e nenhuma inscrição em torneio que não começou; inativo não cria nem reabre convite e não é inscrito; torneio que volta a não ter começado (desfazer o início) não fica, no commit, com inscrito inativo.
- **Desistência e W.O. automático** (migration `20261031010000_withdrawals`): desistência só com o torneio em andamento, só de quem tem seed, e não se desfaz nem muda de data; torneio que volta a não ter começado não fica com desistente. O W.O. automático não tem placar, nem quem enviou, nem quem confirmou, e o perdedor desistiu ou saiu da comunidade; o evento `auto_walkover` tem só o vencedor e nenhum autor. Os checks trocados de `match_results` e `match_events` (eram `match_results_check6` e `match_events_check2`, nomes gerados pelo Postgres) ganharam nomes explícitos.
- **Owner**: exatamente um por tenant, e ele é membro. "No máximo um" vale na hora (índice único parcial); "pelo menos um", no commit. Transferir a posse é, numa transação, rebaixar o owner atual e depois promover o novo (na ordem inversa, o índice recusa).
- Quem inscreveu, quem marcou o check-in e quem criou o convite são membros do mesmo tenant (FK composta); o convite gravado numa inscrição é um link do mesmo torneio.
- **Estado e forma também no banco**: data de início fora do rascunho; status check-in só com janela; tradução nunca no idioma original; ao menos uma fase por torneio; vagas nunca abaixo dos inscritos; slug do tenant e slug, tenant e idioma original do torneio fixos; JSON de regras de campeões e de fase com a forma do `engine` (bans de 0 a 3, melhor de N ímpar); username no formato do cadastro, senha só como hash argon2id, tokens como SHA-256 de 32 bytes, avatar só por https, nome exibido não vazio, campeão com nome único.
- Torneios publicados antes de existir a data de início ganharam a data de criação, e os que estavam na inscrição ficaram com a abertura automática do check-in suspensa (como numa reabertura), para não mudarem de status sozinhos.
- **Fica fora do banco, de propósito**: ordem entre horários de ações de transações diferentes (cada uma grava o próprio `now()`, e uma pode ter começado antes da outra); validade de sessão e de convite posterior à criação e histórico imutável (os testes envelhecem essas datas direto no banco); o limite de 3 tenants por owner e o pool de campeões (regras da API, que mudam sem migration).

## Interface

Decididas na revisão de uso de 28/10 (todas assumidas, sem rodada de perguntas; design por utilidade, sem enfeite).

- **Home**: sem login, diz o que fazer (entrar ou criar conta; link de convite leva ao cadastro). Logado, "Seus torneios" e depois "Minhas comunidades". O status da API só aparece quando há problema.
- **Seus torneios** (rodada 2b, `GET /me/tournaments`): os torneios em que a pessoa está inscrita, de todas as comunidades, cada um com a **situação** dela (inscrição feita, check-in pendente ou feito, fora da chave sem check-in, na chave, terminou em Nº, desistiu, 1º lugar) e, para quem ainda joga, a **próxima partida** como link, com o adversário (ou "quem vencer C1.2") e o que ela espera (enviar o mapa, confirmar ou contestar, aguardar). Os que não encerraram vêm primeiro (em andamento, check-in, inscrição; pelo início); os **encerrados** ficam numa lista à parte, menor, só com os 5 de início mais recente (assumido: cada um refaz a chave para dar a colocação, e a lista inteira cresceria sem fim). As confirmações por tempo vencidas valem antes, como ao abrir a chave. A situação é uma etiqueta (destaque para o check-in que falta) e a próxima partida fica fora do link do torneio, para não haver link dentro de link.
- **Página da comunidade**: torneios, membros, convites e, por último, "Você nesta comunidade" (apelido e sair, ou transferir a posse).
- **Página do torneio**, de cima para baixo: para a organização, um quadro só com o estado, o próximo passo (publicar, abrir o check-in, iniciar, desfazer) e editar; depois a inscrição e o check-in de quem vê (inclusive o aviso de quem ficou fora da chave sem check-in); "Partidas que esperam você" (enviar, responder, resolver) com link; a chave, que ocupa a largura à direita da coluna de texto; descrição, formato, seeding, inscritos, links de inscrição e regras. Encerrado, "Editar" some (nada mais muda).
- **Partida** (refeita para o envio mapa a mapa): o código curto (C2.1) aparece logo abaixo do título, como no cartão da chave; no topo, o placar da série (o dos mapas confirmados enquanto ela corre, com quantos mapas vencem e quantos esperam confirmação ou estão contestados). Depois, **uma linha por mapa**, com o mapa, quem venceu e os frags, uma etiqueta de estado (aguardando, contestado, confirmado) e a frase do estado: enviado por quem e até que horas confirma sozinho, com a contagem; contestado por quem e o comentário; confirmado e como. Confirmar e contestar ficam na linha do mapa; para a organização, "confirmar agora", "aceitar como enviado" e "resolver com outro resultado"/"corrigir" (o formulário abre na própria linha). O **próximo mapa** é a última linha: para quem joga, o formulário aberto (vencedor com os dois nomes como botões de opção lado a lado, o mapa e os frags opcionais); para a organização que não joga, "lançar mapa N" fechado atrás de um botão. O W.O. na série fica no quadro da organização, fechado.
- **Publicar pede confirmação**: publicado, o torneio não volta a rascunho nem pode ser excluído.
- **Seeding**: o título diz quantos inscritos entram na chave (no check-in, quantos fizeram); sem seed nenhum, a lista avisa que a ordem mostrada não vale até salvar (iniciar assim sorteia).
- **Confirmação na própria tela, não `confirm()`** (rodada 2a de UX, 28/10): o botão vira a pergunta com o que vai acontecer, "sim" com o mesmo texto do botão e "Não" (`ConfirmButton`; "Cancelar" se confundiria com "Cancelar inscrição"). Pedem confirmação: publicar, iniciar sem check-in, excluir rascunho, sair da comunidade, remover membro, tirar do torneio, desmarcar check-in de alguém, marcar desistência, dar W.O., cancelar a própria inscrição, revogar link de convite ou de inscrição, remover tradução e sortear o seeding quando a ordem foi mudada. Desfazer o início e transferir a posse continuam pedindo para digitar o slug. O "sim" é vermelho quando a ação apaga ou tira algo; nas que só avançam (publicar, iniciar, W.O., sortear), tem a cor do botão principal.
- **Carregamento**: toda página que busca dado mostra, na hora, o lugar do conteúdo (título e linhas cinza pulsando) com o texto "Carregando…"; todo botão que chama a API fica desabilitado e com um indicador girando até a resposta.
- **Links e texto**: item de lista que abre página é a linha inteira, com o nome sublinhado na cor de link; links soltos e botões-link sempre sublinhados. Texto secundário numa cor própria com contraste AA (no lugar de opacidade). As regras e os motivos estão em "Princípios de interface" no `web/CLAUDE.md`.
- **Página que pede login** oferece "Entrar" e "Criar conta", os dois voltando para a página: quem chega por convite ou link compartilhado geralmente ainda não tem conta.

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

- Duelo: sequência do veto (ex.: ban-ban-pick-pick-decider) e tempo do cronômetro.
- Lista de mapas e pool de duelo (a atual é provisória; ver "Resultados").
- Critérios de desempate (FFA por pontos e grupos).
- Rating para FFA e times (Elo par a par? média do time?).
- Substitutos em times.
