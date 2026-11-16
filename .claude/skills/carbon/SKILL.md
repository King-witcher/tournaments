---
name: carbon
description: Orientação do Carbon Design System v11 (React) destilada para o Telefrag; use em qualquer trabalho de interface em web/ (página, componente, formulário, tabela, gráfico, aviso), antes de consultar o MCP do Carbon.
---

# Carbon no Telefrag

O que o Carbon v11 recomenda (quando usar cada componente, qual variante, hierarquia, texto, acessibilidade, celular), já filtrado pelo que o projeto usa. Serve para escolher o componente e o padrão de uma tela sem gastar uma consulta ao MCP (cada `docs_search` custa milhares de caracteres) e sem reinventar o que o Carbon já resolveu.

A skill descreve o Carbon. O que o projeto decidiu fica nos arquivos dele; aqui só aparece como **"No projeto: ..."**, com o arquivo onde está, quando o Carbon recomenda outra coisa ou quando a `shared/ui` já embrulha o componente.

## Ordem de consulta

1. **O que o projeto já tem.** `web/CLAUDE.md` (biblioteca, princípios, o que não fazer), o JSDoc do componente em `web/src/shared/ui/<nome>.tsx` (props e regras de uso), `docs/design.md` (o que cada página é) e, para o motivo de uma escolha, `docs/decisions/interface.md` e `docs/decisions/tournaments-ui.md`. Componente sempre de `@/shared/ui/<nome>` ou `@/shared/ui/carbon/<pasta>`.
2. **Esta skill**, para o que o projeto ainda não usa ou para conferir uma escolha contra o Carbon.
3. **O MCP `ibm-carbon-design`** só para o que nenhum dos dois cobre: um padrão novo, a especificação exata de um componente (medidas, tokens de estado), código de um gráfico (`get_charts`). Filtro `component_type: "React"` no `code_search`; para gráfico, só `get_charts`.

Na dúvida sobre uma prop, manda o `.d.ts` instalado: `web/node_modules/@carbon/react/lib/components/<Componente>/`.

## Referências

Leia só o arquivo do tema da tela.

- `references/layout.md`: 2x Grid (colunas, breakpoints, gutters), escala de espaçamento, tipografia produtiva e expressiva, página de recurso, UI Shell, breadcrumb.
- `references/color-and-themes.md`: os quatro temas, camadas (`Layer`, tokens contextuais), tokens de texto, borda e suporte, estados de interação, contraste.
- `references/data-display.md`: Data table, Structured list, Contained list, Tile e Tag, e como escolher entre eles.
- `references/navigation.md`: Tabs, Content switcher, menu lateral, Pagination e Pagination nav, Link, Overflow menu, Menu button e Combo button.
- `references/feedback-and-status.md`: notificações (inline, toast, actionable, callout), padrão Status indicators, estados vazios, carregamento (esqueleto, loading, inline), Progress bar e Progress indicator.
- `references/forms-and-actions.md`: hierarquia de botões e alinhamento, Modal e confirmações, exclusão, padrão Forms (layout, rótulos, obrigatório, validação, só leitura) e os campos.
- `references/data-visualization.md`: Carbon Charts (ainda não instalado), tipo de gráfico por pergunta, anatomia, paletas, acessibilidade, dashboards e KPI.
- `references/accessibility-and-content.md`: teclado e foco, alvos de toque, rótulos, e as diretrizes de escrita da IBM (sentence case, rótulo de ação, erro, estado vazio, números e datas).

## Regras do projeto que valem sobre qualquer referência

Resumo; a fonte é `web/CLAUDE.md` e `docs/decisions/interface.md`.

- Tema `g10` no claro e `g100` no escuro; a casca é branca no claro e `g90` no escuro (`shared/ui/_shell-zone.scss`). Conteúdo de leitura num `Panel` (já traz o `Layer`); tabela, chave e lista de tiles ficam soltas.
- Nada de `@carbon/react` pelo índice, `@carbon/ibm-products`, `@carbon/labs`, estilo inline, px ou cor soltos, `!important` ou classe `cds--` escrita à mão. CSS próprio só com tokens, num `.module.scss`.
- Utilitarismo: cada estilo tem um motivo de uso; sem gradiente, sombra decorativa, ícone enfeitando ou pictograma. Ícone só onde o Carbon usa.
- `StatusTag` só para a exceção e para achar quem vê numa lista; o esperado é texto; o que pede cuidado sem ser erro é `IconIndicator` `caution-minor`.
- `Callout` para o que carrega com a página; `InlineNotification` para erro e sucesso de uma ação (os dois pelo `Notice`); toast só para "copiado" e avisos que chegam sozinhos.
- Uma ação primária por página; formulário de página com o primário à esquerda; Modal com o primário à direita.
- Abaixo do `md`, controles com 48px (tokens em `globals.scss`); passe o tamanho de desktop.
- Textos em `messages/` com as regras de `web/messages/CLAUDE.md` ("Como escrever" vale mais que a diretriz de conteúdo da IBM quando as duas divergem).

## Manter

Quando uma consulta ao MCP trouxer um padrão que estes arquivos não têm, ou quando uma decisão de interface for tomada com o usuário:

- A **orientação do Carbon** (o que ele recomenda e por quê) entra no `references/` do tema, em poucas linhas, no mesmo formato: quando usar, quando não, a variante, o que decide a escolha. Nada de colar a página.
- A **decisão do projeto** entra em `docs/decisions/interface.md` (ou `docs/decisions/tournaments-ui.md`, se for de torneio), com o motivo e o número que importa; se for regra de código, também em `web/CLAUDE.md` ou no JSDoc do componente.
- A skill não repete decisões do projeto: no máximo uma linha "No projeto: ..." apontando o arquivo. Se uma decisão mudar, corrija a linha que aponta para ela.
- Mantenha `SKILL.md` em até ~120 linhas e cada referência em até ~200; passando disso, divida o tema.
