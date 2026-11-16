# Layout: grade, espaço, tipo e casca

## 2x Grid

A grade divide ou multiplica por dois. Grade fluida (colunas em porcentagem) para conteúdo que deve crescer com a tela: texto editorial, dashboards, imagens, gráficos. Caixas fixas (múltiplos de um tamanho base) para o que deve aumentar em número, não em tamanho: tiles que quebram linha, ícones numa barra.

| Breakpoint | A partir de | Colunas | Margem da grade | Padding da coluna |
| ---------- | ----------- | ------- | --------------- | ----------------- |
| `sm`       | 320 px      | 4       | 0               | 16 px             |
| `md`       | 672 px      | 8       | 16 px           | 16 px             |
| `lg`       | 1056 px     | 16      | 16 px           | 16 px             |
| `xlg`      | 1312 px     | 16      | 16 px           | 16 px             |
| `max`      | 1584 px     | 16      | 24 px           | 16 px             |

A margem externa é fixa dentro de cada breakpoint, mesmo com colunas fluidas. Teste o desenho em cada breakpoint.

**Acima do `max`**, três modelos:

- **Editorial**: largura máxima, grade centrada. Páginas de leitura, marketing.
- **Produto e documentação**: largura máxima, grade alinhada à esquerda (encostada no menu lateral).
- **Alta densidade**: largura toda, colunas acrescentadas de duas em duas.

**Gutters.** O padrão é 32 px entre blocos de texto (16 px de padding de cada lado). Três modos:

- **Wide** (padrão, 32 px): texto e todo componente com rótulo (campos, dropdowns). Campo com rótulo nunca entra no gutter.
- **Narrow** (16 px): containers "pendurados" no gutter para o texto de dentro alinhar com o texto de fora. Pouco usado em formulário (põe o texto no gutter).
- **Condensed** (1 px): peças separadas que formam um todo (dashboard, visão geral, tiles de portal). Os tiles ganham borda de 1 px em `$border-subtle` para o gutter aparecer.

Misturar modos é a regra, não a exceção: um dashboard condensado ainda tem campos em wide.

**React.** `Grid` (props `narrow`, `condensed`, `fullWidth`) e `Column` com `sm`/`md`/`lg`/`xlg`/`max` como número de colunas ou `{ span, offset }`. Grade aninhada (subgrid): `Grid` dentro de `Column`. No Sass, `@include breakpoint(md)` vale do `md` para cima e `breakpoint-down(md)` abaixo dele.

No projeto: `Page` (`narrow`: 8 colunas no `lg`; `wide`: 16), `PageColumns` (principal e estreita, 11 e 5 na home), modelo produto com o menu lateral e `Grid` centrado sem ele (`docs/decisions/interface.md`, "Menu lateral"). `Grid` e `Column` vêm de `@/shared/ui/carbon/grid`.

## Espaçamento

| Token        | px  | Token        | px  |
| ------------ | --- | ------------ | --- |
| `spacing-01` | 2   | `spacing-08` | 40  |
| `spacing-02` | 4   | `spacing-09` | 48  |
| `spacing-03` | 8   | `spacing-10` | 64  |
| `spacing-04` | 12  | `spacing-11` | 80  |
| `spacing-05` | 16  | `spacing-12` | 96  |
| `spacing-06` | 24  | `spacing-13` | 160 |
| `spacing-07` | 32  |              |     |

Faixas, pelo uso que os componentes do Carbon fazem:

- **01 a 04 (2 a 12 px)**: dentro de um componente. Ícone e texto, rótulo e campo, linhas de uma lista compacta, tag e tag.
- **05 a 07 (16 a 32 px)**: entre componentes e dentro de containers. 16 px é o padding de tile, coluna e célula; 24 px entre itens empilhados num Modal; 32 px entre campos de um formulário de página (o número do Carbon).
- **08 a 10 (40 a 64 px)**: entre seções. 48 px antes dos botões de um formulário de página; 64 px entre blocos grandes.
- **11 a 13 (80 a 160 px)**: layout expressivo, faixas de uma página de apresentação.

Princípios: o que está perto é lido como relacionado, e o espaço maior enfraquece a relação; espaço repetido no mesmo padrão diz "peso igual"; mais espaço em volta dá importância (títulos de nível alto têm mais espaço que os de baixo); espaço separa grupos sem precisar de linha. Uma seção pode ser densa; a página inteira, não.

O `Stack` do Carbon aplica a escala (`gap={6}` = `spacing-06`) entre filhos, sem margem nos componentes; orientação vertical ou horizontal.

No projeto: um grupo de campos num `Stack gap={6}`; nas Configurações, `PageFormLayout` (32 px e 48 px); Modal em 24 px (`web/CLAUDE.md`, "Formulário"). Token no Sass: `spacing.$spacing-05`.

## Tipografia

IBM Plex (Sans, Serif, Mono). Dois conjuntos:

- **Produtivo** (sufixo `-01`, títulos fixos): telas de produto, densas, focadas na tarefa. O tamanho não muda com o breakpoint.
- **Expressivo** (sufixo `-02`, títulos fluidos `fluid-heading-*` e `fluid-display-*`): web e editorial, leitura longa, marketing. Os fluidos crescem aos poucos entre breakpoints. **Nunca dentro de um container**; podem aparecer numa tela de produto, fora de caixas, para criar contraste com o produtivo.

Escala produtiva:

| Estilo                       | Tamanho / altura de linha | Peso     | Uso típico                              |
| ---------------------------- | ------------------------- | -------- | --------------------------------------- |
| `heading-compact-01` / `-01` | 14 / 18 (20)              | semibold | título de tile pequeno, cabeçalho de coluna |
| `heading-compact-02` / `-02` | 16 / 22 (24)              | semibold | título de bloco                         |
| `heading-03`                 | 20 / 28                   | regular  | título de seção                         |
| `heading-04`                 | 28 / 36                   | regular  | título de página                        |
| `heading-05`                 | 32 / 40                   | regular  | título grande                           |
| `heading-06` e `-07`         | 42 e 54                   | light    | destaque, números grandes               |
| `body-compact-01` / `body-01`| 14 / 18 e 14 / 20         | regular  | corpo de interface / parágrafo          |
| `body-compact-02` / `body-02`| 16 / 22 e 16 / 24         | regular  | corpo de leitura                        |
| `label-01`, `helper-text-01` | 12 / 16                   | regular  | rótulo de campo, dica, legenda          |
| `code-01`, `code-02`         | 12 e 14, Mono             | regular  | código, identificadores                 |

Pares de altura de linha: `body-compact-01` com `heading-compact-01`; `body-01` com `heading-01`. Mantenha o mesmo conjunto de estilos dentro de uma tarefa, componente ou região; misture produtivo e expressivo só para marcar hierarquia entre regiões.

No projeto: título de página `heading-04` (`PageHeader`), de seção `heading-03` (`Section`), de bloco `heading-02`; corpo `body-01`; metadado e dica `helper-text-01` em `$text-secondary` (`web/CLAUDE.md`, "Hierarquia"). Sempre `@include type.type-style("...")`, nunca classe `cds--type-*`. Fluido só na apresentação (`fluid-heading-05`, `docs/design.md`, "Início").

## Página de um recurso (detalhe ou visão geral)

Do padrão Global header do Carbon e dos produtos da IBM (IBM Cloud):

- **Global x local.** O cabeçalho é global: está em toda tela e leva o que vale para o sistema todo (trocar de contexto, avisos, perfil). O menu lateral é local: a navegação dentro de um produto ou área, e muda de uma área para outra.
- **Drill-down.** Abrir um objeto a partir de uma lista leva a uma página focada nele, com a trilha (breadcrumb) acima do título para voltar ao nível de cima.
- **Topo da página**: trilha, título, estado e metadado curto, e as ações (uma primária, as outras secundárias, o resto num overflow). Embaixo, as seções do recurso: abas, se forem poucas e da mesma página; menu lateral, se forem páginas com endereço próprio.
- **Visão geral**: blocos de resumo independentes, o mais importante em cima e à esquerda; grade condensada quando os blocos formam um quadro único.
- **Ordem do código = ordem visual**: cabeçalho, navegação, conteúdo, rodapé, para teclado e leitor de tela.

No projeto: o roteiro de toda página ("onde estou", "o que é isto", "o que posso fazer", "minha situação", seções) está em `docs/design.md`, "O que cada página responde"; `PageHeader`, `Section`, `Panel`.

## UI Shell

- **Header**: largura toda, 48 px de altura, fixo no topo ou rolando junto. Nome do produto à esquerda; ações globais (48 x 48) à direita. Em telas pequenas, os links do cabeçalho vão para o menu lateral, no topo dele. O shell aceita outro tema numa zona própria.
- **Left panel (SideNav)**: navegação secundária, abaixo do cabeçalho e fixa à esquerda; mistura links e submenus. Abaixo do `lg` (ou com zoom perto de 175%) fica atrás do botão de menu no início do cabeçalho, abre por cima do conteúdo e o botão vira X. Variante rail: estreita, abre com hover ou foco.
- **Right panel (HeaderPanel)**: painel aberto por uma ação do cabeçalho (perfil, avisos, trocar de contexto); um por vez.
- `SkipToContent` como primeiro item focável.

No projeto: `AppHeader`, `AppSideNav` (+ `AppSideNavLink`, `AppSideNavTitle`), `HeaderPanelAction`, `HeaderSwitcher`, `UserPanel`; as decisões em `docs/decisions/interface.md`, "Menu lateral" e "UI Shell conferido contra o Carbon".

## Breadcrumb

- Mostra onde a página está na hierarquia e leva de volta para cima. Sempre secundária: nunca substitui a navegação principal.
- **Não use** em produto de navegação de um nível só (vira ruído) nem para um processo em passos (é o Progress indicator).
- A página atual, se aparecer, não é link. Trilha longa recolhe os níveis do meio num ⋯ (overflow), que fica na ordem do Tab.
- Tamanhos `sm` e `md`. Links alcançados por Tab e ativados por Enter.

No projeto: só em página que o menu lateral não lista, com a barra depois da mãe e sem o Início; nas seções, só abaixo do `lg` (`docs/decisions/interface.md`, "Trilha só onde ajuda"); `BreadcrumbOverflow` no celular.
