# `shared/ui` por dentro

Como a biblioteca é feita e como ela cresce. O uso (catálogo, formulário, princípios de interface) está em `web/CLAUDE.md`.

## Componentes do Carbon

- O JS do Carbon vem de `src/shared/ui/carbon/<pasta>.ts`: um módulo `"use client"` por pasta do Carbon (`grid`, `stack`, `ui-shell`...), que reexporta os componentes de `@carbon/react/es/...` e liga as feature flags do v11 (`@carbon/react/es/feature-flags.js`; sem elas o `Grid` volta à grade flex). Nunca do índice `@carbon/react`: ele é `"use client"` com efeito colateral declarado, e o Turbopack o põe inteiro (0,8 MB, flatpickr e downshift incluídos) em toda página que o importa; o Biome recusa (`noRestrictedImports`), menos tipos (`import type { ComboBoxProps } from "@carbon/react"`). Componente que ainda não está lá: acrescente o `export` no módulo da pasta (o caminho de cada nome está em `node_modules/@carbon/react/es/index.js`) ou crie o módulo copiando um vizinho. Um módulo por pasta, não um índice só: o Server Component que importa de um módulo `"use client"` leva para o navegador tudo o que os Server Components usam dele.
- `src/app/globals.scss` importa o Sass só das pastas do Carbon que a `shared/ui` usa (as dependências entre elas o próprio Sass do Carbon traz). Componente novo do Carbon: acrescente a pasta dele (`node_modules/@carbon/styles/scss/components`), em ordem alfabética; faltando, ele aparece sem estilo na tela e no Storybook. O `@use "@carbon/react"` inteiro não serve: traz as pastas sem uso e as do `@carbon/ibm-products` (side-panel, tearsheet...).
- O `.scss` passa por um `sass-loader` próprio (`turbopack.rules` em `next.config.ts`): o embutido do Turbopack não acha os `@use` relativos do Carbon. Não volte para o embutido.
- Todo componente do Carbon tem o seu aqui, mesmo os que o site ainda não usa (história no Storybook); ficam de fora só os `preview__`/`unstable__` e as versões Fluid dos campos (o invólucro e a pasta do Sass dela entram quando um formulário pedir).
- Componente novo ou mudado cumpre sozinho os "Princípios de interface" do `web/CLAUDE.md` (foco, `pending`, toque, celular, confirmação, texto traduzido) e diz no JSDoc as props, um exemplo e as regras de uso: é o que a feature lê antes de usar.

## Storybook

`bun run storybook` (http://localhost:6006) mostra cada componente da `shared/ui` nas variações que ele expõe, com tema (`white`/`g100`), idioma e largura (`Celular (390px)`, `md`) na barra de ferramentas e a aba Accessibility (axe) para contraste. `bun run build-storybook` gera o site estático em `storybook-static/` (ignorado).

- A história fica ao lado do componente: `button.tsx` → `button.stories.tsx`, CSF3 como em `button.stories.tsx`. Título `"<Grupo>/<Componente>"`, com os grupos de "Biblioteca `shared/ui`" do `web/CLAUDE.md` (Página, Ação, Formulário, Listas, Estado e conteúdo; mais Cabeçalho e Data e hora).
- Uma história por variação do Carbon que o componente expõe (consulte `code_search` no MCP) e por estado próprio (`pending`, inválido, travado), mais uma que põe as variações lado a lado quando comparar ajuda. Componente que muda no celular ganha uma com `globals: { viewport: { value: "phone" } }`.
- O conteúdo de exemplo pode ser texto em português direto na história (não passa pelo messages); o idioma da barra troca só os textos dos componentes.
- Modal, toast e outro elemento fixo: `parameters: { docs: { story: { inline: false, height: "500px" } } }`, senão cobre a página de documentação.
- Componente novo ou com variação nova na `shared/ui` ganha a história no mesmo commit. Componente de feature fica de fora: depende de dado da API.
- O Storybook usa o Vite, não o Next: `next/navigation`, `next/link`, `next/image` e `next/font` são simulados pelo `@storybook/nextjs-vite`, e o Sass do Carbon lê o `loadPaths` de `.storybook/main.ts`. Componente async (Server Component que busca dado) não roda lá.
