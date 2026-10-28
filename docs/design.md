# Interface: releitura com o Carbon Design System

Alvo da experiência de uso do site, decidido em 28/10/2026 para o MVP. Este arquivo diz **o que** cada página é e **quais padrões** valem em todas; o **como** (instalação, biblioteca `shared/ui`, regras de código) fica em `web/CLAUDE.md`. Motivos curtos em "Interface (Carbon)" no `decisions.md`.

## Decisões de base

- **Carbon v11** (`@carbon/react`, `@carbon/icons-react`), fonte IBM Plex (Sans e Mono) servida pelo `next/font`, sem CDN.
- **Temas**: `white` no claro e `g100` no escuro, seguindo `prefers-color-scheme` até a pessoa escolher outro no painel dela (claro, escuro ou do navegador). Toda tela é conferida nos dois.
- **Só Carbon**: Tailwind sai. Layout com `Grid`/`Column`/`Stack`/`Layer`; o pouco de CSS próprio (chave, avatar, placar) fica em `.module.scss` usando tokens do Carbon (`@use "@carbon/styles/scss/spacing"`, `theme`, `type`, `breakpoint`). Nada de `style={{}}`, nada de pixel ou cor solta.
- **Idiomas**: pt e es continuam; todo texto nos `messages/`, agora um arquivo por namespace (`messages/<locale>/<Namespace>.json`), para fluxos paralelos não disputarem o mesmo arquivo.
- **Os princípios de uso de antes continuam valendo** (carregamento com esqueleto, `pending` sem `disabled`, foco depois de agir, 44px de toque no celular, 390px sem rolagem horizontal fora da chave, estado vazio com próximo passo, erro traduzido junto do controle). Eles estão em `web/CLAUDE.md`; só a forma muda.

## O que cada página responde

Toda página, nesta ordem, de cima para baixo:

1. **Onde estou**: `Breadcrumb` (Início › Comunidade › Torneio › Partida) e o título no `PageHeader`.
2. **O que é isto**: etiquetas (`Tag`) de estado e um metadado curto (formato, data, inscritos N/máx).
3. **O que posso fazer agora**: **uma** ação primária à direita do título (inscrever-se, fazer check-in, publicar, iniciar…), as secundárias ao lado; o que é raro ou destrutivo num `OverflowMenu` (⋯).
4. **Minha situação** (quando há): um `Tile` curto logo abaixo do cabeçalho (inscrito, check-in até X, sua próxima partida).
5. **O resto, em abas** (`Tabs` ligadas a sub-rotas): uma coisa por aba, cada aba com o próprio `loading.tsx`. Nada de página que lista três coisas uma embaixo da outra.

## Padrões que valem em todas

- **Listas grandes** (inscritos, membros, convites, torneios da comunidade): `DataTable` do Carbon com busca na barra de ferramentas, **paginação no cliente** (25 por página; a API não pagina e as listas são limitadas pelo torneio), coluna de estado com `Tag`, ações por linha num `OverflowMenu` (organização) e estado vazio dizendo o próximo passo. No celular (até `md`), só a coluna principal e o estado; as outras somem por CSS (sem rolagem horizontal).
- **Listas "minhas"** (seus torneios, suas comunidades): `ClickableTile` em grade (nome, etiqueta de situação, próxima partida), não tabela.
- **Formulários**: `Form` com `FormGroup` por seção (legenda), `Stack gap={6}`, campos do Carbon com `labelText`, `helperText`, `invalid`/`invalidText`; erro geral em `InlineNotification` acima dos botões; botões no fim em `ButtonSet` (primário "Salvar", secundário "Cancelar"). Campo travado: o motivo em `InlineNotification` (kind `info`) **antes** dos campos.
- **Confirmação**: `Modal` (kind `danger` quando apaga ou tira algo; primário normal quando só avança). O texto diz o que vai acontecer; o botão primário tem o mesmo nome do botão que abriu; "Não" é o secundário; ao fechar, o foco volta ao botão que abriu. Desfazer início e transferir posse continuam pedindo o slug digitado (dentro do mesmo `Modal`).
- **Estado**: `Tag` com mapa fixo de cor por significado (definido na biblioteca): rascunho `gray`, inscrições `blue`, check-in `purple`, em andamento `green`, encerrado `cool-gray`, cancelado/desistiu/saiu `red`, aguardando `teal`, contestado `magenta`, confirmado `green`.
- **Ciclo de vida do torneio**: `ProgressIndicator` (Rascunho → Inscrições → Check-in → Em andamento → Encerrado) com o passo atual; só leitura para quem joga, e na aba Organização ele vem com o próximo passo como botão.
- **Carregamento**: `SkeletonText`/`SkeletonPlaceholder`/`DataTableSkeleton` com a forma do conteúdo final; `InlineLoading` no botão pendente.
- **Mensagens**: `InlineNotification` para erro e aviso na página; `ToastNotification` só para "copiado" e afins.
- **Texto longo** (descrição, regras): `Markdown` com os tokens de tipo do Carbon, em coluna estreita.
- **Larguras**: `Page width="narrow"` (texto e formulário: 8 colunas centradas em `lg`, 16 abaixo) e `width="wide"` (tabela e chave: 16).

## Mapa de páginas

O que ganhou página própria e por quê: **pessoa na comunidade** (quem é o adversário, o que já jogou aqui; hoje não existia), **seeding** (editar a ordem de 64 inscritos pede a tela inteira), **chave** e **partida** (já eram). O que ficou como aba ou bloco: check-in (é um botão e um prazo, vive em "Sua situação" e na tabela de inscritos), links de inscrição e convites (tabelas curtas), configurações da comunidade (três controles). Fora do MVP: perfil global de uma pessoa entre comunidades (apelido e rating são por comunidade) e página pública de comunidade para quem não é membro.

### Casca (todas)

`Header` do UI Shell: `SkipToContent`; `HeaderName` "Torneios QC" (vai ao início); na `HeaderGlobalBar`, idioma (ação com ícone `Translate` abrindo um painel pt/es) e a pessoa (avatar/nome abrindo `HeaderPanel` com "Conta" e "Sair"); sem sessão, "Entrar" e "Criar conta". No fim do painel, com ou sem sessão, o tema (`ContentSwitcher` de ícones: claro, do navegador, escuro). `Content` embaixo com `Grid`. Sem `SideNav` no MVP (as comunidades são pequenas e a maioria abre pelo celular).

### Início `/`

- Sem sessão: título, uma frase do que o site faz, botões "Entrar" (primário) e "Criar conta" (secundário). Convite leva ao cadastro.
- Com sessão: "Seus torneios" (tiles: nome, comunidade, situação, próxima partida como link separado) e "Minhas comunidades" (tiles + "Criar comunidade"). Encerrados numa `Accordion` fechada ("Encerrados", os 5 mais recentes).

### Entrar `/login`, Criar conta `/signup`

Coluna estreita, `Tile` com o formulário (usuário/senha) e o botão do Discord; "Você já entrou como X" quando há sessão. `return_to` continua.

### Conta `/account`

`PageHeader` "Sua conta". Abas: **Perfil** (nome exibido), **Logins** (Discord e senha: ligar, remover), **Senha** (trocar).

### Convite `/invite/[token]`

Coluna estreita: `Tile` com o nome da comunidade, quem convida, validade e "Entrar na comunidade" (ou "Criar conta"/"Entrar" se não há sessão).

### Nova comunidade `/new`

Formulário curto (nome, slug com prévia do endereço).

### Comunidade `/[tenant]`

`PageHeader`: nome, `Tag` com seu papel, ação primária "Novo torneio" (organização). Abas por sub-rota:

- **Torneios** (`/[tenant]`): `DataTable` (nome, formato, estado, início, inscritos). Jogador vê só os publicados.
- **Membros** (`/[tenant]/members`): `DataTable` (membro com avatar, papel, desde), busca; a linha abre a página da pessoa; owner: trocar papel e remover no `OverflowMenu`.
- **Convites** (`/[tenant]/invites`, organização): `DataTable` dos links (criado por, validade, estado), "Gerar link" primário, copiar e revogar por linha.
- **Configurações** (`/[tenant]/settings`): seu apelido; "Sair da comunidade"; owner: transferir a posse (Modal com slug).

`members`, `invites` e `settings` entram na lista de slugs reservados de torneio na API (`tournaments/slug.rs`), como `new` já está.

### Pessoa na comunidade `/[tenant]/members/[user]`

Só para membros da comunidade (`GET /tenants/{slug}/members/{user_id}`). `PageHeader`: avatar, apelido, `Tag` do papel, "membro desde"; se for você, ação "Editar apelido" (leva a Configurações). Depois, **Torneios nesta comunidade**: `DataTable` (torneio, estado, situação da pessoa com `Tag`, colocação) com os mesmos códigos de situação de "Seus torneios"; vazio: "ainda não jogou aqui". Rating entra nesta página quando existir. O apelido é link para cá em toda lista (inscritos, membros, chave, partida, classificação).

A rota já existe: devolve `nickname`, `display_name`, `avatar_url`, `role`, `joined_at` e `tournaments[]` com `tournament` (o `TournamentSummary` da lista da comunidade; `registered`/`checked_in` ali são de quem vê, não da pessoa), `situation` (os mesmos códigos de "Seus torneios") e `placement` (`{rank, tied}` ou nulo). Só os torneios que quem vê também veria na lista da comunidade, mais recentes primeiro, sem limite. `404 member.not_found` para quem não é (ou não é mais) membro.

### Novo torneio `/[tenant]/new` e Editar `/[tenant]/[tournament]/edit`

Um formulário, coluna estreita, `FormGroup` por seção: Identificação (nome, slug, visibilidade), Formato (tipo, eliminação, melhor de, mapas, campeões), Inscrições e datas (máximo, início, check-in, contestação), Textos (descrição e regras em pt, tradução es). Na edição, o aviso dos campos travados antes da seção travada. Rodapé com Salvar/Cancelar; excluir rascunho fica no `OverflowMenu` do `PageHeader` do torneio, não aqui.

### Torneio `/[tenant]/[tournament]`

`PageHeader`: breadcrumb (Início › Comunidade), nome, `Tag` de estado, metadado (formato · melhor de · início · inscritos N/máx). Ação primária conforme quem vê: jogador "Inscrever-se" / "Fazer check-in" / "Cancelar inscrição" (esta no ⋯); organização o próximo passo (Publicar / Abrir check-in / Iniciar), "Editar" secundário, ⋯ com "Reabrir inscrições", "Desfazer início", "Excluir rascunho". Abaixo, o `Tile` "Sua situação" (quem joga). Abas:

- **Visão geral** (`/`): `ProgressIndicator` do ciclo; "Partidas que esperam você"; descrição (Markdown); resumo do formato (`StructuredList`).
- **Chave** (`/bracket`, a partir do início): a chave em largura cheia (o único lugar com rolagem horizontal) e, abaixo, a classificação.
- **Inscritos** (`/participants`): `DataTable` (seed, apelido com avatar, estado: inscrito / check-in feito / desistiu / saiu, inscrito em), busca, paginação; organização: "Inscrever alguém" na barra e, por linha, marcar/desmarcar check-in, tirar, marcar desistência.
- **Regras** (`/rules`): Markdown. Sem regras, a aba não aparece.
- **Organização** (`/manage`, organização): o quadro de estado com o próximo passo e o motivo quando não dá (mínimo de inscritos, check-in aberto); resumo do seeding (definido ou "será sorteado") com o link "Editar seeding"; links de inscrição (tabela, gerar, revogar); W.O. e correções ficam na partida.

### Seeding `/[tenant]/[tournament]/seeding`

Organização, antes do início. Página inteira: título com quantos entram na chave, aviso quando a ordem mostrada ainda não vale, a lista com campo de posição por linha (Enter leva para lá; setas para o ajuste fino), "Sortear" e "Salvar" no fim (`ButtonSet`). Volta para a aba Organização ao salvar.

### Partida `/[tenant]/[tournament]/matches/[match]`

Breadcrumb (… › Torneio › Chave). Cabeçalho: código (C2.1), os dois lados com avatar e o placar da série em tipo grande, `Tag` de estado, prazo com contagem. Depois, **os mapas**: uma linha por mapa (`StructuredList`: nº, mapa, vencedor, frags, `Tag` do estado, frase do estado, ações). O próximo mapa é a última linha, com o formulário aberto para quem joga. Organização: `Accordion` fechada "Organização" (confirmar agora, corrigir, anular, W.O.) e "Histórico" fechado no fim. As regras de texto da rodada de contestação (`decisions.md`, "Interface") continuam.

### Não encontrado e erro

Coluna estreita, título, uma frase e o link de volta (`Link` do Carbon). Os códigos da API continuam traduzidos em `ApiErrors`.

## Biblioteca `web/src/shared/ui`

O que a base entrega (um componente por arquivo, kebab-case; `"use client"` só onde o Carbon precisar):

- Casca e página: `app-header` (Header do UI Shell recebendo os slots idioma e pessoa), `page` (`Page width`), `page-header` (breadcrumb, título, tags, metadado, ações, ⋯), `page-tabs` (abas como links de sub-rota, com a atual marcada), `section` (título de seção + conteúdo), `empty-state`.
- Ação: `button` (Carbon `Button` + `pending` com `InlineLoading`, `aria-disabled`), `button-link` (botão que é `Link` do `i18n/navigation`), `link` (Carbon `Link` sobre o `Link` do i18n), `confirm-button` (abre `Modal`; `danger`; devolve o foco), `overflow-actions` (`OverflowMenu` com itens e confirmação).
- Formulário: `text-input`, `text-area`, `select`, `number-input`, `date-time-input` (datetime-local com visual do Carbon), `checkbox`, `radio-group`, `form-error` (`InlineNotification`), `form-actions` (`ButtonSet`).
- Listas: `data-table` (cabeçalhos, linhas, busca, paginação no cliente, linha-link, `OverflowMenu` por linha, colunas que somem no celular, esqueleto, vazio), `tile-grid` (grade de `ClickableTile`), `structured-list`.
- Estado e retorno: `status-tag` (mapa de cor por significado), `notice` (`InlineNotification`), `toast`, `progress-steps`, `skeleton` (texto, bloco, tabela, página), `accordion`.
- Conteúdo: `avatar` (tokens do Carbon), `markdown`, `countdown`, `local-date-time`, `local-time`, `locale-switcher`, `user-panel` (slot da pessoa no header; o conteúdo vem de `features/auth`).

Os princípios de uso (foco, pending, toque, celular, confirmação) ficam dentro desses componentes, para cada fluxo não os reimplementar.

## Divisão do trabalho (agentes)

Base (um agente, primeiro): instalação, temas, fonte, casca, biblioteca acima, `messages/` por namespace, guia em `web/CLAUDE.md`, Início migrado como exemplo. Depois, em paralelo, cada fluxo é dono de um conjunto de arquivos e não mexe nos dos outros:

| Fluxo | Páginas | Pastas |
| --- | --- | --- |
| A | Entrar, Criar conta, Conta, Convite, Não encontrado | `features/auth`, `features/invites` (só `invite-view`, `accept-invite-button`, `invite-membership`), rotas `login`, `signup`, `account`, `invite`, `[...rest]`, `not-found` |
| B | Comunidade (4 abas), Pessoa na comunidade, Nova comunidade | `features/tenants`, `features/members`, `features/invites` (convites da comunidade), rotas `[tenant]/page`, `members`, `members/[user]`, `invites`, `settings`, `new` (tenant); slugs reservados na API |
| C1 | Torneio (cabeçalho, Visão geral, Inscritos, Regras, Organização), Seeding | `features/tournaments` (menos formulários), `features/participants`, `features/lifecycle`, `features/invites` (links de inscrição), rotas `[tournament]/page`, `participants`, `rules`, `manage`, `seeding`, `layout` |
| C2 | Chave e Partida | `features/matches`, rotas `[tournament]/bracket`, `matches/[match]` |
| D | Novo torneio, Editar torneio | `features/tournaments` (`tournament-form*`, `*-fields`, `translation-*`, `new-tournament-view`, `edit-tournament-view`, `lib/form-values`), rotas `[tenant]/new`, `[tournament]/edit` |

Ponto de contato entre C1 e C2: a Visão geral renderiza `<PendingMatches>` de `features/matches` como está; C2 o restila.
