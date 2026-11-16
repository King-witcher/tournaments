# Exibir dados: tabela, listas, tiles e tags

## Qual escolher

| Conteúdo                                                                  | Componente      |
| ------------------------------------------------------------------------- | --------------- |
| Muitos itens com várias colunas, que se buscam, ordenam ou recebem ação    | Data table      |
| Poucos dados simples em colunas, só leitura (resumo, rótulo e valor)       | Structured list |
| Lista de uma coluna dentro de container, barra lateral ou popover, com ação por item | Contained list |
| Texto em itens (passos, regras), sem estrutura de dado                     | List (`UnorderedList`/`OrderedList`) |
| Objetos para escolher ou abrir, com imagem ou mais de uma linha            | Tile            |
| Rótulo curto que classifica, filtra ou marca um item                       | Tag             |

Lista simples não serve para dado complexo, ordenação, filtro ou seleção (é a Data table). Structured list não aninha itens; com conteúdo complexo, Data table; em espaço pequeno, Contained list.

## Data table

**Quando**: organizar e mostrar dados para examinar e agir sobre eles; ordenar colunas, expandir linhas, agir sobre uma linha ou várias. **Quando não**: como planilha, ou quando a interação com o dado é mais complexa do que linhas e colunas. Pede pelo menos três colunas.

**Tamanhos de linha** (cabeçalho sempre da mesma altura da linha):

| `xs` 24 px | `sm` 32 px | `md` 40 px (padrão) | `lg` 48 px | `xl` 64 px |
| ---------- | ---------- | ------------------- | ---------- | ---------- |

`xs` e `sm` para telas densas; `lg` quando a interação precisa de espaço; `xl` só para linhas com duas linhas de texto. A barra de ferramentas e a de ação em lote acompanham a altura da linha. Use o mesmo tamanho em toda a tela.

**Lugar**: na área principal, com a maior largura possível, sem truncar. Nada de tabela dentro de tabela ou em container pequeno. Pode ficar em qualquer modo de gutter; na grade condensada, separe-a do resto com fundo diferente.

**Partes**:

- **Título e descrição** acima, opcionais.
- **Toolbar**: busca, filtros, configurações da tabela e o botão primário (à direita), que segue a regra de um primário por tela.
- **Busca**: filtra as linhas enquanto se digita. Recolhida (ícone que abre) quando a busca é secundária; aberta quando é o jeito principal de achar algo.
- **Ordenação**: clicar no cabeçalho alterna crescente, decrescente e sem ordem; uma coluna por vez. Cabeçalho ordenável alcançado por Tab e ativado por Enter ou Espaço.
- **Linha expansível**: mostra mais sobre o item sem sair da tabela (progressive disclosure). O chevron vem antes da caixa de seleção.
- **Seleção**: caixa por linha e "selecionar todas" no cabeçalho (este com estado indeterminado); com algo marcado, a toolbar vira a barra de ação em lote (`$background-brand`). Seleção única: radio na primeira coluna, e a ação na direita da toolbar (primário, ghost ou ícone).
- **Ação por linha**: no fim da linha, num Overflow menu, ou botões ghost quando são poucas e frequentes.
- **Paginação** embaixo (simples ou com itens por página e total).
- **Hover da linha** sempre ligado, mesmo sem a linha ser interativa: ajuda a acompanhar a linha com o olho.
- **Carregando**: o esqueleto da tabela (`DataTableSkeleton`), não um spinner.

**Cabeçalhos de coluna**: uma ou duas palavras, sentence case; título longo quebra em duas linhas e depois trunca, com o texto inteiro numa dica.

**Linha que leva a uma página**: o Carbon não tem "linha-link"; o costume nos produtos da IBM é o nome do item como link na primeira coluna.

No projeto: `DataTable` de `shared/ui` com busca, paginação no cliente (25 por página), `href` por linha (a linha inteira abre a página e o nome fica sublinhado na cor de link), colunas `secondary` escondidas abaixo do `md`, ações no `OverflowActions`, `TableSkeleton` e estado vazio no lugar da tabela (`web/CLAUDE.md`, "Listas"; `docs/decisions/interface.md`, "Listas grandes"). `Pagination` avulsa para o mesmo desenho fora da tabela.

## Structured list

Linhas e colunas simples, só para ler, sem ordenação nem ação por linha. A variante selecionável deixa escolher uma linha (feature flag). Cabeçalhos de coluna curtos; o texto da linha quebra em vez de truncar.

No projeto: `StructuredList` (só leitura), por exemplo o resumo do formato na Visão geral, com `InfoTip` no termo técnico.

## Contained list

Lista de uma coluna com título, dentro de algo maior: card, barra lateral, painel.

- **On-page**: num contexto que fica na tela (card, barra lateral).
- **Disclosed**: num contexto temporário (popover, painel elevado).
- Aceita ação por item (fechar, remover), linha clicável, ícone não interativo e busca no topo para listas longas. Divisórias com recuo opcional.

No projeto: `ContainedList` (lista simples com ação por item); a lista do painel do cabeçalho segue o painel direito do UI Shell.

## Tile

Container flexível para informação, orientação ou escolha. Pode ser um container comum com links e botões dentro **ou** ser ele mesmo interativo, nunca as duas coisas.

- **Base**: só um container. As ações de dentro são alvos próprios, cada uma com seu Tab.
- **Clickable**: o tile inteiro é um alvo, uma ação só (em geral abrir uma página). Nada interativo dentro dele. A seta (ou o ícone de ir) no canto diz que ele navega.
- **Selectable**: escolha que precisa de mais que um rótulo. Seleção única (como radio) ou múltipla (como checkbox); faz parte de um formulário.
- **Expandable**: mostra o resumo e abre o resto no lugar; com ou sem elementos interativos na parte aberta.
- Tiles numa grade usam as proporções do 2x Grid; em grade condensada, borda `$border-subtle`. Tile é mais simples que um "card" (que, no Carbon for IBM Products, junta mídia, ações e metadado).

No projeto: `LinkTile` + `TileGrid` (listas "minhas", com capa e logo opcionais), `MetaLines`, `RadioTileGroup` (com `columns`), `SelectableTile`, `ExpandableTile`, e `Tile` direto de `@/shared/ui/carbon/tile` para o container base (o cartão da partida, "Sua situação").

## Tag

Rótulo curto para classificar, categorizar, filtrar ou marcar um item com uma propriedade.

- **Read-only**: só marca (categoria, propriedade, estado).
- **Dismissible**: o que pode sair, como um filtro aplicado ou um item escolhido num campo; o X tira.
- **Selectable**: liga e desliga, como filtro.
- **Operational**: abre algo relacionado (um popover com os itens de um grupo).
- **Não use** tag como link para outra página ou para tirar a pessoa da tela; não dê duas funções à mesma tag.
- Tamanhos `sm`, `md`, `lg`. Texto curto, sentence case; longo, trunca com dica. Cores por significado (vermelho, magenta, roxo, azul, ciano, teal, verde, cinzas, alto contraste, outline), sempre com o mesmo significado no produto; a cor não é a única pista.

No projeto: a etiqueta é só para a exceção e para achar quem vê numa lista; o esperado é texto (`docs/decisions/interface.md`, "Etiqueta só na exceção"; `docs/design.md`, "Estado"). `StatusTag` com quatro cores fixas (vermelho, magenta, roxo, cinza), `DismissibleTag`, `OperationalTag`, `SelectableTag`.
