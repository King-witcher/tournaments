# Navegação: abas, alternadores, paginação, links e menus

## Menu lateral, abas ou content switcher

| Situação                                                                    | Componente        |
| --------------------------------------------------------------------------- | ----------------- |
| Várias páginas de uma área, cada uma com endereço próprio (navegação local) | Menu lateral (UI Shell left panel) |
| Seções diferentes do mesmo objeto, no mesmo nível, sem sair da página        | Tabs              |
| Vistas diferentes do mesmo conteúdo, no mesmo lugar (lista e grade, por rodada e por confronto) | Content switcher |
| Um processo linear em passos                                                | Progress indicator |
| Ligar ou desligar uma opção                                                 | Toggle            |

As abas seguem a metáfora do arquivo de pastas: duas abas não guardam a mesma folha. O content switcher fica num nível abaixo, muitas vezes dentro do conteúdo de uma aba, para reorganizar o que ela mostra.

## Tabs

**Quando**: separar conteúdo relacionado em vistas do mesmo nível, que a pessoa consulta uma de cada vez. **Quando não**:

- trocar o formato ou filtrar o mesmo conteúdo: Content switcher;
- guiar um passo a passo: Progress indicator;
- comparar informação de grupos diferentes: abas obrigariam a ir e voltar; ponha lado a lado.

**Variantes**:

- **Line** (padrão): no nível da página, logo abaixo do cabeçalho; a linha marca a aba aberta.
- **Contained**: quando a área das abas é um container (o painel colado nelas), ou dentro de componentes; aceita rótulo secundário (uma contagem, um estado curto). Tamanhos `sm` 32, `md` 40, `lg` 48 px.
- **Vertical**: muitas seções numa área larga, com a lista à esquerda.

**Conteúdo**: rótulos de uma ou duas palavras, sentence case, que não quebram; muitas abas rolam com botões de seta. Abas só com ícone levam dica. Dismissible: abas que a pessoa abre e fecha (documentos abertos).

**Teclado**: Tab entra na aba ativa; as setas andam entre as abas. Ativação automática (a seta já troca o painel) quando o conteúdo abre rápido; manual (a seta só move o foco, Enter ou Espaço troca) quando abrir custa.

No projeto: o menu lateral é a navegação de cada área (`docs/decisions/interface.md`, "Menu lateral" e "Conta com menu lateral"); `PageTabs` para abas que são sub-rotas onde não há menu; `Tabs` com o `TabList` de `shared/ui` (sem as setas de rolagem no celular) para conteúdo da mesma página, como as fases na Chave e na Classificação.

## Content switcher

**Quando**: alternar entre duas ou mais vistas de conteúdo parecido no mesmo espaço; uma de cada vez; a troca vale na hora. Pode ser binário de vista (grade ou lista).

**Quando não**: navegar entre subpáginas (abas); escolher sim ou não, ligado ou desligado como valor de formulário (Toggle ou Radio).

- Uma opção sempre escolhida; a primeira, se nada indicar outra.
- Alto contraste (o escolhido em `$background-inverse`) ou baixo contraste (`lowContrast`, o escolhido num fundo claro).
- Variante só com ícones (vistas), com o nome na dica. Tamanhos `sm`, `md`, `lg`. Rótulos curtos e do mesmo tamanho.

No projeto: `ContentSwitcher` em baixo contraste por padrão (o preto do alto contraste pesava mais que o título), de duas a cinco opções, `Switch` ou `IconSwitch` (JSDoc em `shared/ui/content-switcher.tsx`).

## Pagination e Pagination nav

- **Pagination**: a de tabela e listas de dados. Itens por página, intervalo ("1–25 de 230"), página atual e setas; versão simples (só página e setas) ou completa. Embaixo da tabela, com a altura da linha.
- **Pagination nav**: páginas de conteúdo (resultados, artigos, galeria): os números das páginas, perto do que pagina, com largura pelo número de páginas; recolhe o meio num ⋯. Tamanhos `sm`, `md`, `lg`.
- **Carregar mais** (padrão Loading): estende uma lista curta em lotes, quando a ordem de leitura importa mais que pular para uma página.

No projeto: a paginação do `DataTable` no cliente (25 por página; `Pagination` avulsa para o mesmo desenho); `PaginationNav` para páginas de conteúdo; as notificações chegam ao rolar, em páginas de 20 (`docs/design.md`, "Casca").

## Link

**Quando**: navegar para outro lugar (outra página, outro site, uma seção da mesma página). **Quando não**:

- ação que muda dado, estado ou a forma de exibir: é botão;
- imagem como link: use um tile, que leva a imagem e o resto.

**Variantes**:

- **Standalone**: sozinho ou logo depois de um conteúdo; sem sublinhado; pode levar um ícone (seta de ir, ícone de abrir fora).
- **Inline**: dentro de frase ou parágrafo; sublinhado; sem ícone.

Texto do link diz o destino; "saiba mais" genérico precisa de contexto ligado a ele para o leitor de tela. Muitos links poluem a página e escondem o principal. Estilo de visitado é opcional. Tamanhos `sm`, `md`, `lg`.

No projeto: `Link` de `shared/ui` (solto, azul, com 48 px de altura no celular; `inline` no meio do texto), `ButtonLink` para a ação que leva a outra página com cara de botão, `LinkTile` e a linha do `DataTable` com `href` para item de lista (`web/CLAUDE.md`, "Link"). Nunca `next/link` nem `<a>` para página interna.

## Overflow menu (⋯)

**Quando**: ações a mais, quando falta espaço; em geral numa linha de tabela, num card ou ao lado das ações de uma página. **Quando não**: para a ação principal (fica à vista) ou para uma ação só (vira botão).

- Sempre um botão ghost com o ícone ⋯; aberto, o menu usa a cor da camada.
- Item destrutivo por último, em vermelho (danger), separado por uma divisória.
- Itens com verbo, curtos; desabilitado só quando a razão está clara ao lado.
- Teclado: Tab chega; Enter ou Espaço abre com o foco no primeiro item; setas andam; Esc fecha e devolve o foco ao botão.

No projeto: `OverflowActions` (com `confirm` para o item destrutivo, que abre o Modal; uma linha antes do primeiro destrutivo) e `ContextMenu` para o botão direito.

## Menu button e Combo button

- **Menu button**: várias ações sob um botão com nome ("Exportar", "Adicionar"), quando as ações se aplicam ao mesmo objeto e mostrar todas ocuparia espaço. Pode ser primário, tertiary ou ghost. Mais de três ações sobre o mesmo dado devem ir para um menu, não para uma fileira de botões.
- **Combo button**: a ação principal à vista e, na seta ao lado, as variações dela. Para quando falta espaço e uma das ações pesa mais; costuma ficar no topo da página. Só no estilo primário. Duas paradas de Tab (o botão e a seta).
- Ações distintas sobre o mesmo objeto, sem uma principal: menu button. Uma principal com alternativas: combo button. Ações secundárias e raras: overflow.

No projeto: `MenuButton` e `ComboButton` (JSDoc em `shared/ui/menu-button.tsx` e `combo-button.tsx`); no topo da página, uma primária, as secundárias ao lado e o raro no ⋯ (`docs/design.md`, "O que cada página responde").
