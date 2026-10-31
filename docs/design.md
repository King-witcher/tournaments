# Interface: o que cada página é

Alvo da experiência de uso, no Carbon Design System. Este arquivo diz **o que** cada página é e **quais padrões** valem em todas; o **como** (biblioteca `shared/ui`, regras de código, textos) fica em `web/CLAUDE.md`, e os motivos em `decisions/interface.md` e `decisions/tournaments-ui.md`.

## Base

- Carbon v11, fonte IBM Plex pelo `next/font`, só tokens do Carbon (sem Tailwind, `style={{}}`, pixel ou cor soltos). Temas `white` e `g100`, seguindo o sistema até a pessoa escolher no painel dela; toda tela é conferida nos dois.
- Idiomas pt e es, um arquivo de mensagens por namespace.
- Princípios de uso (carregamento com esqueleto, `pending`, foco depois de agir, 48px de toque no celular, 390px sem rolagem horizontal fora da chave, estado vazio sem ensinar o uso, erro traduzido junto do controle) em `web/CLAUDE.md`.

## O que cada página responde

Toda página, de cima para baixo:

1. **Onde estou**: `Breadcrumb` (Início › Comunidade › Torneio › Partida) e o título no `PageHeader`.
2. **O que é isto**: `Tag`s de estado e um metadado curto (formato, data, inscritos N/máx).
3. **O que posso fazer agora**: **uma** ação primária à direita do título, as secundárias ao lado; o raro ou destrutivo num `OverflowMenu` (⋯).
4. **Minha situação** (quando há): um `Tile` curto logo abaixo do cabeçalho.
5. **O resto, em abas** (`Tabs` ligadas a sub-rotas): uma coisa por aba, cada uma com o próprio `loading.tsx`.

## Padrões

- **Listas grandes**: `DataTable` com busca, paginação no cliente (25 por página), coluna de estado com `Tag`, ações por linha num `OverflowMenu` (organização); até `md`, só a coluna principal e o estado. **Listas "minhas"** (seus torneios, suas comunidades): `ClickableTile` em grade.
- **Formulários**: `Form` com `FormGroup` por seção, `Stack gap={6}`, erro geral em `InlineNotification` acima dos botões, botões no fim (primário "Salvar", secundário "Cancelar"). Campo travado: o motivo em `InlineNotification` (kind `info`) **antes** dos campos.
- **Confirmação**: `Modal` (kind `danger` quando apaga ou tira algo); texto diz o que vai acontecer, primário com o nome do botão que abriu, "Não" secundário, foco volta ao gatilho.
- **Estado**: `Tag` com cor fixa por significado: rascunho `gray`, inscrições `blue`, check-in `purple`, em andamento `green`, encerrado `cool-gray`, cancelado/desistiu/saiu `red`, aguardando `teal`, contestado `magenta`, confirmado `green`.
- **Ciclo do torneio**: `ProgressIndicator` (Rascunho → Inscrições → Check-in → Em andamento → Encerrado); só leitura para quem joga, e na aba Organização vem com o próximo passo como botão.
- **Mensagens**: `InlineNotification` para erro e aviso na página; `ToastNotification` só para "copiado" e afins.
- **Texto do organizador** (descrição, regras): texto simples (`UserText`), coluna estreita. **Larguras**: `Page width="narrow"` (texto e formulário) e `"wide"` (tabela e chave).

## Mapa de páginas

Têm página própria: **pessoa na comunidade**, **ordem da chave**, **chave** e **partida**. Ficam como aba ou bloco: check-in (botão e prazo, em "Sua situação" e na tabela de inscritos), links de inscrição e convites (tabelas curtas), configurações da comunidade. Fora do MVP: perfil global entre comunidades e página pública de comunidade.

### Casca (todas)

`Header` do UI Shell: `SkipToContent`; `HeaderName` "Torneios QC"; na `HeaderGlobalBar`, com sessão, suas comunidades (painel com a lista, a atual marcada e "Criar comunidade"), o sininho (número das que faltam ver; painel com os avisos dos últimos 30 dias), o idioma (painel pt/es) e a pessoa (avatar e nome abrindo `HeaderPanel` com "Conta", "Sair" e, no fim, o tema); sem sessão, "Entrar" e "Criar conta". Sem `SideNav`: cada nível tem no máximo cinco seções, que cabem nas abas, e a chave precisa da largura toda.

### Início `/`

- Sem sessão: título, uma frase do que o site faz, "Entrar" (primário) e "Criar conta" (secundário).
- Com sessão: "Pendências da organização" (só para quem organiza e tem alguma), "Seus torneios" (tiles: nome, comunidade, situação, próxima partida como link separado; encerrados numa `Accordion` fechada, os 5 mais recentes) e "Minhas comunidades" (+ "Criar comunidade").

### Entrar `/login`, Criar conta `/signup`, links por email

Coluna estreita, `Tile` com o formulário e os botões do Discord e do Google; "Você já entrou como X" quando há sessão; `return_to` continua. Entrar é com email e senha; o cadastro pede nome exibido, email e senha e, depois do envio, mostra "Confira seu email" (endereço canônico, validade de 1 hora, campo do código de 6 dígitos, "Reenviar", "Usar outro email"). "Esqueci a senha" (`/login/forgot`): o campo do email e resposta neutra.

`/signup/confirm`, `/login/reset` e `/email/confirm` leem o token depois do `#`. O cadastro confirma sozinho ao carregar e leva para dentro já logado; a recuperação (senha nova) e a troca de email pedem uma ação, porque os scanners de link abririam a página. Link vencido: `Notice` de erro e o caminho (criar conta de novo, pedir outro link).

### Conta `/account`

Abas: **Perfil** (nome exibido e, no fim, a zona de perigo "Excluir conta"), **Logins** (email primeiro, com a senha atual pedida quando ele muda e "Remover email" travado com o motivo se há senha; Discord e Google cada um no seu bloco, vincular ou desvincular se sobrar outro jeito de entrar; senha), **Senha**, **Sessões** (a atual marcada, encerrar uma ou as outras), **Notificações** (as do navegador e o som).

### Convite `/invite/[token]`, Nova comunidade `/new`

Convite: coluna estreita, `Tile` com o nome da comunidade, o dono, o número de membros e "Entrar na comunidade" (ou "Criar conta"/"Entrar" sem sessão). Nova comunidade: nome e slug com prévia do endereço.

### Comunidade `/[tenant]`

`PageHeader`: nome, `Tag` do papel, "Novo torneio" (organização). Abas por sub-rota:

- **Torneios**: `DataTable` (nome, formato, estado, início, inscritos); jogador vê só os publicados.
- **Ranking** (todos): posição, membro, rating com `Tag` "provisório", mapas, V–D, último mapa; a linha abre a página da pessoa.
- **Membros**: membro com avatar, papel, desde e, para a organização, "Entrou por"; a linha abre a página da pessoa; o owner troca papel e remove no ⋯.
- **Convites** (organização): links (começo do link, criado por, validade, quantos entraram), "Gerar link", copiar e revogar por linha.
- **Configurações**: apelido; owner: rating da comunidade (salvar recalcula todos), Discord; no fim a **zona de perigo** (`DangerZone`, uma seção por ação): sair da comunidade; owner: transferir a posse e excluir a comunidade (Modal com slug; agendada, mostra a data e "Cancelar exclusão"). Com a exclusão agendada, todas as abas têm o aviso com a data (só leitura).

### Pessoa na comunidade `/[tenant]/members/[user]`

Só para membros. `PageHeader`: avatar, apelido, `Tag` do papel, "membro desde"; se for você, "Editar apelido". Depois: **Rating** (resumo em `StructuredList`, "Ajustar rating" para a organização), **Últimos mapas** (a linha abre a partida), **Ajustes manuais** (organização) e **Torneios nesta comunidade** (estado, situação da pessoa e colocação). O apelido é link para cá em toda lista.

### Novo torneio `/[tenant]/new` e Editar `/[tenant]/[tournament]/edit`

Uma página, coluna estreita; à vista nome, id, início, chave e séries; o resto em acordeão com resumo no título. Detalhes do formulário em `decisions/tournaments-ui.md`. Editar tem abas (Geral, Formato, Inscrições e prazos, Resultados) e um Salvar; excluir rascunho fica no ⋯ do torneio.

### Torneio `/[tenant]/[tournament]`

`PageHeader`: breadcrumb, nome, `Tag` de estado, metadado (formato · melhor de · início · inscritos N/máx). Ação primária conforme quem vê: jogador "Inscrever-se" / "Fazer check-in" (cancelar a inscrição no ⋯); organização, o próximo passo (Publicar / Abrir check-in / Iniciar), "Editar" secundário, ⋯ com "Desfazer início" e "Excluir rascunho". Abaixo, "Sua situação". Abas:

- **Visão geral**: `ProgressIndicator`, "Partidas que esperam você", descrição, resumo do formato (`StructuredList` com `InfoTip`) e "Ver as regras".
- **Chave** (a partir do início): a chave em largura cheia (o único lugar com rolagem horizontal) e a classificação. Cada lado do cartão: seed, avatar, apelido e placar; linhas curvas ligam a partida ao lado que quem sai dela ocupa na seguinte. Na dupla, abas **Chave principal** (com a grande final) e **Repescagem**, abrindo na aba da partida de quem vê.
- **Inscritos**: `DataTable` (seed, apelido, estado, inscrito em) com busca; organização: "Inscrever alguém", check-in, tirar, desistência por linha e, depois da tabela, os links de inscrição.
- **Regras** (sempre): "Como funciona" gerado da configuração e "Regras adicionais", editáveis ali pela organização.
- **Organização**: quadro de estado com o próximo passo e o motivo quando não dá, resumo da ordem da chave com "Definir a ordem"; no TDM, o resumo dos times e o link da formação.

### Ordem da chave `/[tenant]/[tournament]/seeding`

Organização, antes do início: quantos entram na chave, a lista com campo de posição por linha e o rating de cada um, "Sortear", "Ordenar por rating" e "Salvar" no fim; volta para a aba Organização.

### Partida `/[tenant]/[tournament]/matches/[match]`

Código (C2.1), os dois lados com avatar e o placar da série em tipo grande, `Tag` de estado, prazo com contagem. Depois, **uma linha por mapa** (`StructuredList`: nº, mapa, vencedor, frags, estado e ações) com o próximo mapa como última linha e, em cada mapa, os screenshots (`features/screenshots`). No veto, o bloco do veto e do draft de campeões ao vivo. Organização: `Accordion` fechada "Organização" (W.O.) e "Histórico" no fim.

### Não encontrado e erro

Todas as páginas de erro são o mesmo desenho (`features/errors/components/error-page.tsx`): coluna estreita, título, o que houve num `Notice` e a saída ("Tentar de novo" quando há `retry`, e o botão para o início); nunca a mensagem do erro, só o `digest` pequeno. Quatro casos, nenhum cai numa página do Next: `[locale]/not-found.tsx`, `[locale]/error.tsx` (dentro do layout), `global-error.tsx` (erro no layout raiz: documento próprio, idioma pelo primeiro segmento) e `not-found.tsx` da raiz (primeiro segmento que não é idioma, como `/api/x`: pt e es, um embaixo do outro). Os dois últimos refazem a casca em `standalone-error-document.tsx`.
