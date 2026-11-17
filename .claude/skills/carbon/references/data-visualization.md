# Visualização de dados: Carbon Charts

## Situação no projeto

Desde 16/11 o projeto tem `@carbon/charts-react` (e `date-fns`, para o idioma do eixo do tempo), num gráfico só: a evolução do rating na página da pessoa (`docs/decisions/interface.md`, "Página da pessoa"). O resto dos números continua em tabela, `StructuredList` e tiles. Para um gráfico novo:

- Pergunte se um gráfico responde a pergunta melhor que a tabela ou o tile que já existem (utilitarismo: o gráfico precisa de um motivo de uso).
- No projeto: `LineChart` de `shared/ui` (`line-chart.tsx`), que carrega o pacote e o CSS por `next/dynamic` só no navegador e só na página que o usa (113 KB de JS e 20 KB de CSS com gzip), com o esqueleto da altura dele, o tema seguindo o do site (`white` no claro, `g90` no escuro, a camada do `Panel`) e as datas e números no idioma da página. Outro tipo de gráfico: um invólucro irmão, no mesmo molde.
- Armadilha: o eixo do tempo formata as marcas com o date-fns (`timeScale.localeObject`), não com `locale.date`/`locale.time`; e, com pontos a menos de 15 segundos uns dos outros, cai no formato do d3 em inglês, com milissegundos (dado de teste confirmado no mesmo segundo).

## Pacote

- Componentes React: `SimpleBarChart`, `GroupedBarChart`, `StackedBarChart`, `LineChart`, `AreaChart`, `StackedAreaChart`, `DonutChart`, `PieChart`, `MeterChart`, `GaugeChart`, `ScatterChart`, `BubbleChart`, `HistogramChart`, `ComboChart`, `RadarChart`, `TreemapChart`, entre outros. `import { GroupedBarChart } from "@carbon/charts-react"`, uso `<GroupedBarChart data={data} options={options} />`.
- **Dados**: lista de objetos `{ group, key, value }` (ou `date` no lugar de `key` em série temporal); `group` é a série, e é ele que recebe a cor.
- **Opções que decidem o uso**: `title`; `axes.left`/`axes.bottom` com `mapsTo` (qual campo) e `scaleType` (`LABELS`, `TIME`, `LINEAR`, `LOG`); `height`; `theme`; `locale` (formatos de número e data); `legend`; `tooltip`; `toolbar`; `color.scale` (grupo para cor) ou `color.pairing` (`option`, `numberOfVariants`: as paletas prontas por número de séries); `data.loading` (o próprio esqueleto); `data=[]` mostra o estado vazio do gráfico; `accessibility.svgAriaLabel`; `tabularRepModal` (a tabela do "mostrar como tabela" da toolbar).
- Código e interfaces: `get_charts` do MCP (`mode: "schema"` e depois `"full"`; `interface_names` para as opções). Nunca `code_search` para gráfico.

## Tipo de gráfico pela pergunta

| Pergunta                                           | Gráfico                                                        |
| -------------------------------------------------- | -------------------------------------------------------------- |
| Como algo mudou no tempo; comparar algumas séries  | **Line** (contínuo); **Step** se o valor muda em momentos discretos; **Area** para volume |
| Comparar categorias discretas                      | **Bar simples**; horizontal com rótulos longos ou em ranking   |
| Comparar subgrupos dentro de cada categoria        | **Bar agrupada** (poucos grupos)                               |
| Parte do todo por categoria, com o total importando | **Bar empilhada** (as partes do meio comparam mal)            |
| Parte do todo num momento, com poucas fatias       | **Donut** (o total no centro) ou **Pie**; com muitas fatias pequenas, bar |
| Um valor contra um limite ou capacidade            | **Meter** (proporcional: as partes de uma capacidade); **Gauge** para um KPI contra a meta |
| Tendência num espaço pequeno, ao lado de um número | **Sparkline** (linha ou área sem eixos, grade nem legenda)     |
| Relação entre duas medidas                         | **Scatter**; **Bubble** com uma terceira no tamanho            |
| Como os valores se distribuem                      | **Histogram** ou **Boxplot**                                   |

## Anatomia

Gráfico retangular: título, eixos, marcas (ticks), título do eixo, legenda, toolbar, barra de zoom, moldura e tooltip. Circular: rótulos e, no centro, o número grande.

- **Título**: descritivo, diz a conclusão que o dado mostra.
- **Legenda**: liga cada cor, forma ou tamanho ao dado. Prefira rótulo direto no gráfico a uma legenda longa.
- **Tooltip**: o detalhe de um ponto, ao passar o mouse ou focar.
- **Eixos**: eixo numérico começa no zero em barra e área (cortar o eixo exagera a diferença). Em linha, começar fora do zero pode ser certo quando a variação é pequena. Nunca mude o intervalo das marcas para acomodar o dado; compressão leva a quebra de eixo desenhada. Série temporal: quando cruza um novo dia, mês ou ano, o rótulo vira marco em semibold. Formatos de número e data no idioma.
- Texto curto e claro em tudo; eixo com a unidade.

## Cores

- **Categórica** (séries sem ordem entre si): aplique a sequência **na ordem**, sem pular; ela foi montada para maximizar o contraste entre vizinhas. Com número de séries previsível (até cinco), use as paletas prontas para aquela quantidade (`color.pairing`). Para conferir os valores, `docs_search` "data visualization color palettes"; o pacote já aplica.
- **Sequencial** (monocromática): magnitude ordenada, do claro ao escuro de uma cor (mapa de calor, intensidade).
- **Divergente**: duas cores com o neutro no meio, para desvio de um ponto central. Paleta 1 (vermelho e ciano) quando há associação com temperatura; paleta 2 (roxo e teal) sem essa associação (desempenho, taxa de variação). As divergentes são as mesmas no claro e no escuro.
- **Alerta**: só para estado: vermelho erro ou perigo, laranja aviso sério, amarelo aviso, verde normal ou sucesso.
- A mesma série tem a mesma cor em todos os gráficos de uma tela.

No projeto: cor de gráfico sai do pacote ou dos tokens; nada de hex no código (`web/CLAUDE.md`, "O que não fazer"). A vitória em teal e a derrota em roxo da tabela cruzada (`docs/design.md`, "Chave") são cor de estado do projeto, não da paleta de gráfico.

## Acessibilidade

- Marcas, linhas e bordas de gráfico com contraste de 3:1 contra o fundo.
- Cor nunca é a única pista: rótulo direto, forma ou padrão, legenda em texto.
- Uma alternativa em tabela (a toolbar tem "mostrar como tabela"); no projeto, a tabela já existente costuma ser essa alternativa, ao lado ou num `ContentSwitcher` (gráfico e tabela).
- `svgAriaLabel` com o que o gráfico mostra; o título e uma frase de conclusão em texto também servem a quem não vê o gráfico.

## Dashboards e KPI

- **De apresentação** (o estado dos indicadores de relance): hierarquia forte, com o mais importante em cima e à esquerda, mais contraste e mais área (leitura em F); poucos indicadores, o resto sob demanda; cores consistentes; espaço em branco para separar e dar respiro.
- **De exploração** (investigar o dado): consistência entre os gráficos, gráficos ligados (filtrar um filtra os outros), anotações nos pontos que importam.
- **Número grande (KPI)**: o valor em destaque, o rótulo do que ele mede e o contexto (período, comparação); a variação com o differential indicator (seta e cor, nunca só a cor).
- Blocos independentes na grade condensada, com borda `$border-subtle`; carregamento progressivo, com esqueletos.

No projeto: não há dashboard; a Visão geral do torneio e a home são telas de trabalho, ordenadas pelo que pede ação (`docs/design.md`). Um bloco de números entraria como seção de uma dessas páginas, não como página nova, salvo decisão com o usuário.
