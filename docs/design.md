# Interface: o que cada página é

Alvo da experiência de uso, no Carbon Design System. Este arquivo diz **o que** cada página é e **quais padrões** valem em todas; o **como** (biblioteca `shared/ui`, regras de código) fica em `web/CLAUDE.md`, os textos em `web/messages/CLAUDE.md`, e os motivos em `decisions/interface.md` e `decisions/tournaments-ui.md`.

## Base

- Carbon v11, fonte IBM Plex pelo `next/font`, só tokens do Carbon (sem Tailwind, `style={{}}`, pixel ou cor soltos). Temas `white` e `g100`, seguindo o sistema até a pessoa escolher no painel dela; toda tela é conferida nos dois.
- Idiomas pt, es e en, um arquivo de mensagens por namespace.
- Princípios de uso (carregamento com esqueleto, `pending`, foco depois de agir, 48px de toque no celular, 390px sem rolagem horizontal fora da chave, estado vazio sem ensinar o uso, erro traduzido junto do controle) em `web/CLAUDE.md`.

## O que cada página responde

Toda página, de cima para baixo:

1. **Onde estou**: `Breadcrumb` (Início › Comunidade › Torneio › Partida) e o título no `PageHeader`.
2. **O que é isto**: `Tag`s de estado e um metadado curto (formato, data, inscritos N/máx).
3. **O que posso fazer agora**: **uma** ação primária à direita do título, as secundárias ao lado; o raro ou destrutivo num `OverflowMenu` (⋯).
4. **Minha situação** (quando há): um `Tile` curto logo abaixo do cabeçalho.
5. **O resto, em abas** (`Tabs` ligadas a sub-rotas): uma coisa por aba, cada uma com o próprio `loading.tsx`.

## Padrões

- **Listas grandes**: `DataTable` com busca, paginação no cliente (25 por página), coluna de estado com `Tag`, ações por linha num `OverflowMenu` (organização); até `md`, só a coluna principal e o estado. **Listas "minhas"** (seus torneios, suas comunidades): `ClickableTile` em grade. **Tarefas** ("Esperando você"): `Tile` com o botão, um por linha.
- **Formulários**: `Form` com `FormGroup` por seção, `Stack gap={6}`, erro geral em `InlineNotification` acima dos botões, botões no fim (primário "Salvar" à esquerda, secundário "Cancelar" ao lado, como o Carbon pede em formulário de página). Campo travado: o motivo num `Callout` (kind `info`) **antes** dos campos.
- **Confirmação**: `Modal` (kind `danger` quando apaga ou tira algo); texto diz o que vai acontecer, primário com o nome do botão que abriu, "Não" secundário, foco volta ao gatilho.
- **Estado**: `Tag` com cor fixa por significado: rascunho `gray`, inscrições `blue`, check-in `purple`, em andamento `green`, encerrado `cool-gray`, cancelado/desistiu/saiu `red`, aguardando `teal`, contestado `magenta`, confirmado `green`.
- **Ciclo do torneio**: `ProgressIndicator` (Rascunho → Inscrições → Check-in → Em andamento → Encerrado); só leitura (nenhum passo tem hover, mão ou foco do Tab), e na aba Organização vem com o próximo passo como botão.
- **Mensagens**: `Callout` para informação e cuidado que carregam com a página (fixos, sem fechar); `InlineNotification` para erro e sucesso, o retorno de algo que aconteceu; `ToastNotification`, no canto de cima à direita, só para "copiado" e para os avisos que chegam sozinhos.
- **Texto do organizador**: descrição em texto simples (`UserText`), regras da organização em Markdown (`MarkdownText`, com o campo `MarkdownField`), coluna estreita. **Larguras**: `Page width="narrow"` (texto e formulário) e `"wide"` (tabela e chave).

## Mapa de páginas

Têm página própria: **pessoa na comunidade**, **ordem da chave**, **chave** e **partida**. Ficam como aba ou bloco: check-in (botão e prazo, em "Sua situação" e na tabela de inscritos), o link de convite e o de inscrição (botão que abre o Modal com o link), configurações da comunidade. Fora do MVP: perfil global entre comunidades e página pública de comunidade.

### Casca (todas)

`Header` do UI Shell: `SkipToContent`; `HeaderName` com o logo do Telefrag (só o ícone abaixo do `md`); na `HeaderGlobalBar`, com sessão, suas comunidades (painel com a lista e "Criar comunidade", que abre a criação num Modal; a atual sem marca visível, só para o leitor de tela), o sininho (o número das que faltam ler no `BadgeIndicator` do Carbon; painel com as não lidas e, atrás de "Ver lidas", as lidas, em páginas de 20 que chegam ao rolar) e a pessoa (avatar e nome abrindo `HeaderPanel` com "Conta", "Sair" e, no fim, o tema, num `ContentSwitcher`); sem sessão, o mesmo painel com "Entrar" e "Criar conta". Um painel aberto por vez. Sem `SideNav`: cada nível tem no máximo seis seções, que cabem nas abas, e a chave precisa da largura toda. No rodapé, no fim da tela mesmo em página curta, os idiomas: um link para cada um e o atual como texto.

### Início `/`

- Sem sessão, a **apresentação** (estática), em faixas de parede a parede que alternam o fundo e a camada: abertura com o título expressivo (`fluid-heading-05`), uma frase, "Criar conta" (primário, leva à criação da comunidade depois do cadastro) e "Entrar", a frase com o link da sala de veto e a captura da chave; "Como funciona" em três passos; "Do veto ao ranking" em abas (mapas, campeões, resultado, ranking), texto e captura, e o link da sala de veto em mapas e campeões; "Para quem organiza" em quatro colunas; e "Criar conta" de novo no fim. As capturas são do próprio site, por idioma e por tema.
- Com sessão (`/me`, servido em `/`): o título "Início", sem frase, e duas colunas (`PageColumns`; abaixo do `lg`, uma embaixo da outra). Na principal: "Esperando você" (some sem nada: um por linha, com o logo quando alguma tem, o torneio, a comunidade, o que é e o botão; primeiro as ações de quem joga, com o check-in feito ali mesmo, depois as da organização); "Seus torneios" (tiles: a capa no topo quando a grade tem alguma, nome, comunidade, o início ou a próxima série com o próximo mapa, e a etiqueta do que a série espera ou da situação; do que espera por quem vê ao que só espera a data; numa `Accordion` fechada, "Participações encerradas"); e "Inscrições abertas" (tiles com a capa: nome, comunidade, início, formato e vagas; some sem nada). Na lateral: "Suas comunidades" (o logo antes do nome quando alguma tem) e, depois da lista, "Criar comunidade" discreto. Sem comunidade nenhuma, só o estado vazio com "Criar comunidade", na principal. Os dois abrem a criação num Modal.

### Entrar `/login`, Criar conta `/signup`, links por email

Coluna estreita, `Tile` com o formulário e os botões do Discord e do Google; "Você já entrou como X" quando há sessão; `return_to` continua. Entrar é com email e senha; o cadastro pede nome exibido, email e senha e, depois do envio, mostra "Confira seu email" (endereço canônico, validade de 1 hora, campo do código de 6 dígitos, "Reenviar", "Usar outro email"). "Esqueceu a senha? Entre por email" leva a `/login/email`: o campo do email, resposta neutra, o campo do código (envia sozinho ao completar) e, recolhida, a senha nova opcional.

`/signup/confirm`, `/login/email/confirm` e `/email/confirm` leem o token depois do `#`. Nenhum confirma ao carregar: o cadastro tem um botão "Confirmar", entrar por email um botão "Entrar", com a senha nova recolhida, e a troca de email um "Confirmar email", porque os scanners de link abririam a página. Link vencido: `Notice` de erro e o caminho (criar conta de novo, pedir outro link).

### Conta `/account`

Abas: **Perfil** (nome exibido, a foto, que abre um Modal para recortar, dar zoom e girar antes de enviar, e, no fim, a zona de perigo "Excluir conta"), **Logins** (email primeiro, com a senha atual pedida quando ele muda e "Remover email" travado com o motivo se há senha; Discord e Google cada um no seu bloco, vincular ou desvincular se sobrar outro jeito de entrar; senha: criar ou, com ela, trocar e "Remover senha", travado com o motivo sem Discord nem Google), **Sessões** (a atual marcada, encerrar uma ou as outras), **Notificações** (as do navegador e o som).

### Convite `/invite/[token]`, Criar comunidade (Modal e `/new`)

Convite: coluna estreita, o logo da comunidade no título, `Tile` (com a capa do torneio no topo, no link de inscrição) com o nome da comunidade, o dono, o número de membros, "Sobre o torneio" (no link de inscrição) e "Sobre a comunidade" (cada uma só se tiver descrição) e "Entrar na comunidade" (ou "Criar conta"/"Entrar" sem sessão). Criar comunidade: nome e endereço com a prévia do link, "Cancelar" e "Criar", que leva à comunidade; o logo fica nas Configurações. Num Modal a partir do cabeçalho e do início; `/new` é o mesmo formulário numa página, para depois do cadastro e o link direto.

### Comunidade `/[tenant]`

`PageHeader`: logo (quando há), nome, `Tag` do papel, "Novo torneio" (organização; abre a criação num Modal) e, quando há, a descrição em até 2 linhas, em todas as abas, com "Ver mais"/"Ver menos" (só aparece se o texto passa de 2 linhas, medido, também ao redimensionar). Abas por sub-rota:

- **Torneios**: "Ativos" em tiles (a capa no topo quando a grade tem alguma, nome, início, modalidade e vagas, a `Tag` do estado e as de quem vê), do em andamento ao rascunho; embaixo, "Encerrados" num `DataTable` (nome, formato, data, inscritos), que some sem nenhum. Jogador vê só os publicados.
- **Ranking** (todos): posição, membro, rating com `Tag` "provisório", mapas, V–D, último mapa; a linha abre a página da pessoa.
- **Membros**: membro com avatar, papel e desde; a linha abre a página da pessoa; o owner troca papel e remove no ⋯. Para a organização, "Convidar" na barra da tabela: o link único, para copiar, e "Gerar novo link".
- **Configurações**: owner: nome e descrição, endereço (com a confirmação do que ele quebra), logo (salva sozinho); apelido; owner: rating da comunidade (salvar recalcula todos), Discord; no fim a **zona de perigo** (`DangerZone`, uma seção por ação): sair da comunidade; owner: transferir a posse e excluir a comunidade (Modal com slug; agendada, mostra a data e "Cancelar exclusão"). Com a exclusão agendada, todas as abas têm o aviso com a data (só leitura).

### Pessoa na comunidade `/[tenant]/members/[user]`

Só para membros. `PageHeader`: avatar, apelido, `Tag` do papel, "membro desde"; se for você, "Editar apelido". Depois: **Rating** (resumo em `StructuredList`, "Ajustar rating" para a organização), **Por mapa** (um tile por mapa jogado, com a imagem, V–D e a porcentagem, do mais jogado ao menos), **Últimas partidas** (a linha abre a série), **Ajustes manuais** (organização) e **Torneios nesta comunidade** (estado, situação da pessoa e colocação). O apelido é link para cá em toda lista.

### Novo torneio (Modal) e Configurações `/[tenant]/[tournament]/settings`

**Novo torneio**: o botão do cabeçalho da comunidade (e o do estado vazio da lista) abre um `Modal` com "Começar de" primeiro (`ComboBox`: vazio é a configuração padrão; um torneio da comunidade traz a configuração dele), nome, endereço (gerado do nome, com a prévia do link) e modalidade (duelo ou TDM, com o tamanho do time); "Cancelar" e "Criar". Cria sempre rascunho e leva às Configurações dele. "Copiar torneio", no ⋯ do torneio, abre o mesmo Modal com ele em "Começar de". `/[tenant]/new` (com `?from=`) é o mesmo formulário numa página, para o link direto. Detalhes em `decisions/tournaments-ui.md`.

**Configurações**: a última aba do torneio, só para a organização (`/edit` leva a ela). Coluna estreita, um `Accordion` com um item por seção, todos fechados ao abrir (vários podem ficar abertos; `#maps` no endereço abre Mapas): no título, o nome e o resumo do que está salvo, com "Mudanças não salvas" (`IconIndicator` de cuidado) quando algo difere do salvo, ou "Só leitura"; dentro, o formulário com o próprio "Salvar" (desabilitado enquanto um campo recusa o que tem; salvo, "Salvo."). Na ordem: **Geral** (nome, endereço com a confirmação do que ele quebra, descrição e, depois do Salvar, a capa, que salva sozinha), **Início e inscrições** (início, inscrição, check-in), **Formato** (modalidade e tamanho, quem age pelo time no TDM logo embaixo, vagas, chaveamento, séries, final, reset, rodadas, rating e, nas fases de tabela, o Desempate), **Mapas**, **Campeões**, **Resultados** (contestação, captura) e **Outros idiomas**. Chaveamento, quem age pelo time e mapa de cada partida são tiles com o que cada opção faz. Nada depende de recarregar: o que vem do salvo de outra seção (as opções do TDM em Campeões, o pool nas séries) muda logo depois que ela salva. O que o estado não deixa mudar fica só para leitura (o padrão "Read-only states" do Carbon), sem o Salvar, com o motivo num `Callout` antes dos campos: o endereço depois de publicar; Formato, Mapas, Campeões, início e check-in depois de começar; encerrado, tudo menos o nome, a descrição, a capa e o nome e a descrição das traduções. A série que pede mais mapas que o pool salvo fica inválida na hora, com "Ir para Mapas"; o erro que só a API vê aparece na seção que salvou, com "Ir para" a seção onde corrigir. As regras da organização ficam na aba Regras; excluir, no ⋯ do torneio.

### Torneio `/[tenant]/[tournament]`

`PageHeader`: breadcrumb, nome, `Tag` de estado, metadado (formato · melhor de · início · inscritos N/máx) e, do `lg` para cima, a capa à direita (2:1). Ação primária conforme quem vê: jogador "Inscrever-se" / "Fazer check-in" (cancelar a inscrição no ⋯); organização, o próximo passo (Publicar / Abrir check-in / Iniciar), ⋯ com "Copiar torneio" (o Modal da criação), "Desfazer início" e "Excluir" (o rascunho; publicado, com o endereço digitado; depois de começar, só o owner). Editar é a aba Configurações. "Publicar" (no cabeçalho e na aba Organização) abre a confirmação com a revisão da configuração salva (nome, início, formato, inscrições, mapas, campeões, resultados, idiomas) e diz que publicado não volta a rascunho. Abaixo, "Sua situação". Abas:

- **Visão geral**: `ProgressIndicator`, "Séries que esperam você" (com o próximo mapa, nas que ele joga), descrição, resumo do formato (`StructuredList` com `InfoTip`) e "Ver as regras".
- **Chave** (a partir do início): a chave em largura cheia (o único lugar com rolagem horizontal). Cada lado do cartão: seed, avatar, apelido e placar; linhas curvas ligam a partida ao lado que quem sai dela ocupa na seguinte. Na dupla, abas **Chave principal** (com a grande final) e **Repescagem**, abrindo na aba da partida de quem vê. No topo de cada cartão, na chave e nas rodadas, o ícone do estado (concluída, em andamento, pendente), com os símbolos dos passos da série; a concluída fica sem preenchimento. No todos contra todos, a de quem vê diz "Adversário jogando" no lugar da tarefa, também no topo da série e em "Seus torneios". No todos contra todos, as séries em vez da grade, e um `ContentSwitcher` alterna **Rodadas** (o padrão) e **Confrontos**, a tabela cruzada: cada célula é o resultado de quem está na linha e abre a série, com o check e o placar sobre teal (vitória) ou roxo (derrota), o parcial sobre cinza com o meio círculo (em andamento) ou só o ícone de pendente; a diagonal fica no fundo da página. No suíço, só as rodadas.
- **Classificação**: no todos contra todos e no suíço, a partir do início, a tabela com pontos, séries (vencidas–perdidas), partidas (fora na md1, porque repetiriam as séries), saldo de frags, Buchholz no suíço (com o (i) no título da coluna) e os desempates embaixo; na eliminação, só depois de encerrar, com as colocações e a série que eliminou cada um.
- **Inscritos**: `DataTable` (seed, apelido, estado, inscrito em) com busca; organização: "Link de inscrição" ao lado do título (o link único, para copiar, e "Gerar novo link"), "Inscrever alguém", check-in, tirar e desistência por linha.
- **Regras** (sempre): "Como funciona" gerado da configuração e "Regras da organização", editáveis ali pela organização.
- **Organização**: quadro de estado com o próximo passo e o motivo quando não dá, resumo da ordem da chave com "Definir a ordem"; no TDM, o resumo dos times e o link da formação.
- **Configurações** (organização): as seções que configuram o torneio, cada uma com o próprio Salvar (acima, em "Novo torneio (Modal) e Configurações").

### Ordem da chave `/[tenant]/[tournament]/seeding`

Organização, antes do início: quantos entram na chave, a lista com campo de posição por linha e o rating de cada um, "Sortear", "Ordenar por rating" e "Salvar" no fim; volta para a aba Organização.

### Partida `/[tenant]/[tournament]/matches/[match]`

Título com o código (C2.1), os dois lados e, depois da primeira partida enviada, o placar entre eles; `Tag` de estado e, embaixo, só quando a série terminou ou não começou, como e quando. Embaixo, os passos (`ProgressSteps` navegável: os feitos e o atual abrem, o atual com o mesmo hover sublinhado e acessível pelo teclado; sem espaço, vertical ou, com mais de 4, a lista num `Dropdown`): Pronto, Mapas e Campeões (com veto no torneio) e uma partida por passo, até o melhor de N, cada uma com o placar ou o estado embaixo do nome. Um passo por tela, com título; abre no que pede ação de quem vê. Mapas e campeões em tiles como o pool do formulário (imagem, nome, etiqueta e o ícone de estado no canto: check no escolhido, X no banido, em cinza; na vez de quem vê, o tile clicável com a ação escrita; o campeão só diz se dá para escolher: banido com o X, o que quem vê não pode escolher apagado com o motivo numa dica, o resto disponível), Campeões com o resumo do draft ao lado (uma coluna por lado e, por partida, campeão, mapa, campeão). Cada partida num cartão (`Tile`: lado A, mapa, lado B, com os campeões, "Você" e "Capitão"; embaixo, os frags de cada lado, no envio ou no resultado), com o estado, as ações e os screenshots. Quem a regra do torneio não deixa agir vê o tabuleiro sem ação e uma frase ("O capitão escolhe pelo time."); o capitão que escolhe por todos marca o colega num `RadioGroup`, com o próximo sem escolha já marcado. Organização: `Accordion` fechada "Organização" (W.O.) e "Histórico" no fim.

### Sala de veto `/veto` e `/veto/[token]`

Sem login. **Criar** (`/veto`, coluna estreita): o modo (duelo, 2v2, 3v3, 4v4), o melhor de N e, nos times, quem age por eles à vista (muda quantos links a sala tem); Mapas e Campeões em acordeão com o resumo no título, com os campos do formulário de torneio. **A sala** (`/veto/[token]`, larga) é uma página só, que se adapta ao link: quem joga entra (logado, com o nome da conta; sem login, digita um nome) e faz os passos da série, Pronto, Mapas e Campeões, com um último, Resultado (o texto para o chat, à vista, "Copiar o resultado do veto" como primário e o resumo do draft); o espectador vê os mesmos passos sem ação; a gestão vê os links dos lugares e o de espectador (`CodeSnippet` e abrir em guia nova, com quem entrou em cada lugar; o dela é o endereço da página) e o andamento em duas colunas (`PageColumns`), com "Começar agora" até começar. Sala vencida ou inexistente: o estado e "Criar outra sala". Sem indexação nem referrer, e a prévia do link é a genérica. Entradas: botão secundário na abertura da apresentação e um tile na coluna lateral do início.

### Não encontrado e erro

Todas as páginas de erro são o mesmo desenho (`features/errors/components/error-page.tsx`): coluna estreita, título, o que houve num `Notice` e a saída ("Tentar de novo" quando há `retry`, e o botão para o início); nunca a mensagem do erro, só o `digest` pequeno. Quatro casos, nenhum cai numa página do Next: `[locale]/not-found.tsx`, `[locale]/error.tsx` (dentro do layout), `global-error.tsx` (erro no layout raiz: documento próprio, idioma pelo primeiro segmento) e `not-found.tsx` da raiz (primeiro segmento que não é idioma, como `/api/x`: pt e es, um embaixo do outro). Os dois últimos refazem a casca em `standalone-error-document.tsx`.
