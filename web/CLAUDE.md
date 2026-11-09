@AGENTS.md

# Frontend (web/)

- Só apresentação: nunca acessa o banco. Os dados vêm da API Rust pelo client `api` (`src/shared/api/client.ts`, openapi-fetch).
- Bun para pacotes e scripts; Biome para lint e formatação (não ESLint/Prettier; confere também os `.scss`). `src/shared/api/schema.d.ts` é gerado: não edite à mão, rode `bun run gen:api`.
- Todo texto visível vem de `messages/<idioma>/<Namespace>.json` (next-intl), nada de texto no JSX; chave nova entra em `pt`, `es` e `en`. Códigos da API (status, erros) são traduzidos aqui.
- O idioma é o segmento raiz `[locale]`; no servidor, `getTranslations` já lê o idioma de `next/root-params` (não passe locale à mão).
- Links e redirects pelo `src/i18n/navigation.ts`, não por `next/link`/`next/navigation`, para manter o prefixo de idioma. Link visível: `Link`/`ButtonLink` de `shared/ui`. Tipo de destino: `Href`, de `@/i18n/navigation`.
- Cache Components está ligado: dado buscado a cada requisição fica dentro de `<Suspense>`.
- Interface: Carbon Design System v11 (`@carbon/react`, `@carbon/icons-react`). O que cada página é está em `docs/design.md`; aqui fica o como.

Duas pastas têm guia próprio, que o Claude Code carrega ao abrir um arquivo delas (leia direto se for criar arquivo lá sem abrir outro antes):

- `messages/CLAUDE.md`: textos (namespaces, termos da interface, "Como escrever"). Vale para todo texto novo ou mudado.
- `src/shared/ui/CLAUDE.md`: a biblioteca por dentro (módulos do Carbon, Sass, componente novo, Storybook). Para só usar a biblioteca, basta este arquivo.

## Organização

```
src/
  app/[locale]/...      só rotas: lê params, monta <Page>, <Suspense> e componentes de feature
    (home)/             a apresentação (/ sem sessão), estática
    me/                 o início de quem entrou, servido em / pelo proxy (src/proxy.ts)
  app/globals.scss      Carbon (lista de componentes), temas, fonte, toque, foco
  features/<domínio>/   tudo de um domínio, os mesmos da API
    components/         um componente por arquivo (+ o .module.scss com o mesmo nome)
    server.ts           busca no servidor usada por mais de um componente
    hooks/  lib/        hooks e funções puras do domínio
  shared/               serve a qualquer domínio; nunca importa de features/
    ui/  hooks/  api/  lib/
  i18n/                 next-intl: routing, navigation, request, namespaces, provider do cliente
messages/<pt|es|en>/    um JSON por namespace
```

- Componente que conhece um domínio (chama a API, usa tipo do schema, tem texto próprio) fica em `features/<domínio>/components/`; em `shared/ui` só entra o que serviria a qualquer domínio. Nada de `components/`, `utils/` ou `helpers/` soltos em `src/`: se não sabe onde pôr, falta nomear o domínio.
- `page.tsx` não busca dado nem tem lógica: compõe. A busca fica no componente de feature. Feature pode usar componente de outra, sem ciclo; o que duas precisam em comum e é genérico vai para `shared`.
- Arquivo em kebab-case com o nome do que exporta (`member-row.tsx` → `MemberRow`), um componente exportado por arquivo, estilo ao lado em `member-row.module.scss`. Import relativo dentro da feature, `@/features/...` ou `@/shared/...` fora. Sem `index.ts` reexportando (mistura servidor e cliente e atrapalha o bundle).

### Dados e mutações

- Server Component por padrão; `"use client"` só na folha que precisa de estado ou evento (o botão, não a seção).
- Busca: componente async que busca o próprio dado, dentro de `<Suspense>`; busca usada em mais de um lugar vai para `features/<domínio>/server.ts`, com `cache()` do React. No servidor, sempre por `serverApi` (leva o segredo que livra o Next do banimento de scanners; `stack.md`, "Scanners"), nunca pelo `api`.
- Busca em série: o `layout.tsx` e a `page.tsx` rodam juntos no servidor (o `children` do layout é outra parte da resposta), mas os filhos que um componente devolve só começam depois do `await` dele. Componente que espera uma busca antes de devolver os filhos começa antes a deles: `preload(getX(...))` (`shared/lib/preload.ts`), com os mesmos argumentos do uso e só para o que o filho buscaria de qualquer jeito (modelo: `HomeSessionGate`).
- Mutação de botão: `useApiAction` (`run(chamada, aoDarCerto)` com `pending` e `error` já traduzido). Formulário: `useActionState` + `callApi`. Nunca mostre o código cru da API.
- Server Component não passa função para Client Component (`onClick`, `onSelect`, `renderIcon`, `as`). Lista do servidor com ação por linha: o servidor monta as linhas e põe em cada uma um componente de cliente da feature (`<MemberActions member={m} />`).

### Tempo real

- Página que se atualiza sozinha: `<LiveUpdates>` (`features/realtime`) dizendo o que ela mostra (`tenant`, `tournament`, `match`, `me`), no `params.then(...)` da página ou do `layout.tsx` do grupo. Sem ele, só muda ao navegar. O refresh mantém o `useState` dos componentes de cliente; `key` derivada do dado remonta o bloco.
- O que recarrega cada página: `features/realtime/lib/interest.ts`. Conexão, toasts e som: `features/realtime`; Web Push e service worker: `features/push` e `app/sw.js/route.ts`; sininho: `features/notifications`.
- Ação cuja resposta a página aplica sozinha, sem `router.refresh()`: mande `headers: useRealtimeHub()?.originHeaders()`, e a mudança de partida que ela causar não volta a esta aba pelo socket (as outras abas recebem).

## Estilo (Carbon)

- Componente do Carbon vem de `@/shared/ui/carbon/<pasta>` (`import { Stack } from "@/shared/ui/carbon/stack"`), nunca do índice `@carbon/react`, que põe o Carbon inteiro em toda página (o Biome recusa; `import type` pode). Faltando o componente lá, ou o estilo dele: `src/shared/ui/CLAUDE.md`.
- Tema `g10` (fundo cinza, containers brancos) em `:root` e `g100` no escuro (do sistema ou `data-theme="dark"`). A casca (cabeçalho e menu lateral) é branca no claro e `g90` no escuro (`shared/ui/_shell-zone.scss`, a zona de tema do Carbon em CSS), e o conteúdo de leitura vai num `Panel`; em quem pode ficar dentro de um `Panel`, token de camada (`$layer`, não `$layer-01`).
- Toque: abaixo de `md` (672px), os controles `sm` e `md` valem 48px por token em `globals.scss`. Passe o tamanho de desktop (`md`, o padrão; `sm` em linha de tabela); nenhum componente trata.
- O pouco de CSS próprio (grade, alinhamento, avatar, placar, chave) vai num `.module.scss` ao lado do componente, só com tokens: `@use "@carbon/react/scss/spacing"` (`spacing.$spacing-05`), `theme` (`theme.$text-secondary`), `type` (`@include type.type-style("heading-03")`), `breakpoint` (`breakpoint(md)` é "do md para cima"; `breakpoint-down(md)` é o celular). Utilitários em `@carbon/react/scss/utilities/` (`visually-hidden`, `skeleton`, `z-index.z("modal")`).
- Para mudar uma propriedade que o próprio Carbon põe no elemento (o padding do `Content`), ponha um elemento seu em volta ou dobre a classe (`.panel.panel { padding: 0; }`), que tem o mesmo peso.

### O que não fazer

- `style={{…}}`; px, rem ou cor soltos (a única medida fora de token é a borda de `1px`); `!important`.
- Classe `cds--…` escrita à mão ou `:global(.cds--…)`: use o componente, uma prop dele ou um módulo seu.
- Tailwind (saiu), `@carbon/labs` e `@carbon/ibm-products`.
- `next/link`, `Link` do Carbon com `href` cru, `<a>` para página interna: use `Link`/`ButtonLink` de `shared/ui`.
- `Modal`/`ComposedModal` do Carbon direto numa feature: use `ConfirmButton`, `ConfirmModal`, `FormModal` ou `PassiveModal`.
- Texto em inglês que o Carbon põe por padrão (aria-label "Close", "Clear search input"): os componentes de `shared/ui` já traduzem; usando um do Carbon direto, passe a prop traduzida.

## Biblioteca `shared/ui`

Cada componente já cumpre os princípios abaixo (foco, `pending`, toque, celular, confirmação): use-o em vez do Carbon cru. Props, exemplos e regras estão no JSDoc de `src/shared/ui/<nome>.tsx`; leia antes de usar. Todo componente do Carbon tem o seu aqui (menos os `preview__`/`unstable__` e as versões Fluid dos campos). Layout sem regra própria (`Grid`, `Column`, `Stack`, `Layer`, `Tile`, `Form`, `FormGroup`, `SkeletonText`, `InlineLoading`, `AccordionItem`, `SelectItem`, `UnorderedList`/`ListItem`) vem direto do Carbon, de `@/shared/ui/carbon/<pasta>`, também em Server Component, enquanto as props forem serializáveis.

Os que não se deduzem do nome:

- **Página**: `Page` (`narrow` para texto e formulário, `wide` para tabela e chave), `PageColumns` (numa `wide`, a coluna principal e a estreita de atalhos), `Panel` (o container branco de texto, formulário e lista de rótulo e valor, um por assunto, sobre a página cinza; tabela, chave, lista de tiles e as grades do veto ficam soltas), `PageHeader` (topo de toda página: trilha, título com o logo, tags, metadado, ações e ⋯, e a capa à direita no `lg`), `AppSideNav` (o menu lateral de uma área, num `layout.tsx`, com `AppSideNavLink` e o topo `AppSideNavTitle`; com ele, o `Page` encosta no menu), `PageTabs` (abas que são sub-rotas, no `layout.tsx` do segmento, onde não há menu lateral), `TabList` (no lugar do do Carbon, dentro de `Tabs`: sem as setas de rolagem no celular).
- **Ação**: `ButtonLink` (ação que leva a outra página), `ConfirmButton` (ação que não se desfaz fácil), `FormModal` (formulário curto num Modal), `ActionWithNote` (o motivo de não dar, ou o erro, embaixo do botão), `OverflowActions` (o ⋯), `MenuButton` (vários comandos atrás de um botão com nome), `ComboButton` (o primário e, no menu ao lado, as variações dele), `ContextMenu` (botão direito), `CopyButton`.
- **Formulário**: `FormSection`, `PageFormLayout` (o espaçamento das páginas de configurações: 32px entre os campos e 48px antes dos botões), `FormRow` (campos curtos lado a lado, em colunas que seguem a largura do bloco: 4 na página larga, 2 num Modal, 1 no celular; um campo curto sozinho numa linha larga vai nele para não esticar, e `columns={2}` em volta de um formulário o deixa na largura de leitura), `RequiredLabel`, `TextInput`, `PasswordInput`, `TextArea`, `MarkdownField` (o `TextArea` em Markdown, com as abas Escrever/Visualizar e, com `onImage`, "Inserir imagem"), `Select` (nativo, poucas opções), `NumberInput`, `DatePicker` (uma data), `DateRangePicker`, `TimePicker`, `DateTimeInput` (data, hora e fuso), `Dropdown` (lista do Carbon), `ComboBox` (lista longa: digita e filtra), `MultiSelect` (várias), `CheckboxGroup`, `RadioTileGroup` (com `columns`, os tiles em fileira, como o Carbon põe tiles na grade) e `SelectableTile` (escolha que precisa de mais que um rótulo), `ContentSwitcher` (vistas do mesmo conteúdo), `Slider`, `Search`, `FileUploader`, `FileDropZone`, `FormError`, `FormActions`. Erro por campo: `FormResult`, `invalidProps`, `formError`, `fieldId` (`shared/lib/form-result.ts`) e `useFocusInvalid`.
- **Listas**: `DataTable` (grande, com busca e paginação), `Pagination` (a do `DataTable`, avulsa), `PaginationNav` (páginas de conteúdo, perto dele), `TileGrid` + `LinkTile` (as listas "minhas", com logo e capa opcionais), `ActionTile` (uma tarefa com o botão que a resolve, em `TileGrid` `stacked`), `MetaLines` (o metadado de um tile em mais de uma linha), `ExpandableTile`, `StructuredList` (só leitura, em colunas), `ContainedList` (lista simples com ação por item), `TreeView`.
- **Estado e conteúdo**: `StatusTag`, `DismissibleTag`, `OperationalTag`, `SelectableTag`, `IconIndicator`/`ShapeIndicator`, `Notice` (o padrão: `Callout` para informação e cuidado que carregam com a página, `InlineNotification` para erro e sucesso), `Callout`, `ActionableNotification` (aviso que pede uma resposta), `Toast` ("copiado"), `ToastStack` (avisos que chegam sozinhos), `Loading` (a tela toda espera), `ProgressBar`, `UserText` (texto de usuário sem formatação, com as quebras de linha), `MarkdownText` (texto de usuário em Markdown, sem HTML: as regras da organização), `InfoTip` (o (i) ao lado de um termo que nem todos conhecem), `Tooltip`, `DefinitionTooltip`, `Popover`, `CodeSnippet`, `AILabel`, `LocalDateTime`/`LocalDate`/`LocalTime`/`LocalDayTime` ("hoje às 21h" no meio de uma frase), `Countdown`, `TimeAgo`.

Formulário: `Form` com `action` do `useActionState`, um `FormSection` por grupo (com um grupo só, os campos direto num `Stack gap={6}`; nas Configurações, no `PageFormLayout`), erro de campo no `invalidText` (`invalidProps`), `FormError` acima dos botões, `FormActions` no fim. Campo travado: `Notice kind="info"` com o motivo **antes** dos campos. Modelo: `features/tenants/components/create-tenant-form.tsx`. Validação: "Formulários: validação" em Princípios de interface; `useFormValidity` (`shared/hooks/`) e os filtros de `shared/lib/input-filters.ts`.

Campos de torneio (as seções da aba Configurações, a criação curta e a sala de veto): cada bloco é `memo(Componente, sameValues(KEYS))` (`features/tournaments/lib/same-values.ts`) e tipa `values` como o `Pick` dos campos que lê; o `onChange` e os `errors` de `useSettingsSection` e `useNewTournament` não mudam de objeto à toa, e função passada a um bloco também não pode mudar (nada de `onChange={(x) => ...}` inline nele). Sem isso, cada tecla renderiza a seção inteira. Só leitura (o torneio já começou): `readOnly` nos blocos, não `disabled`.

## Quando quebrar um componente

- passou de ~100 linhas;
- busca dado e também desenha a lista: separe a lista do item (`MemberList` + `MemberRow`);
- tem mais de uma seção com título: uma seção por componente;
- item de lista com mais de ~10 linhas de JSX: `<XRow>` ou `<XTile>`;
- estado que só uma parte usa (ex.: "copiado"): componente da parte;
- lógica com estado que se repete: hook (em `shared/hooks` se for genérica).

## Princípios de interface

Utilidade antes de enfeite: cada estilo precisa de um motivo de uso (hierarquia, affordance, legibilidade, toque, foco, leitura no celular); "fica bonito" não é motivo. Nada de gradiente, sombra decorativa, ícone enfeitando card ou emoji; ícone só onde o Carbon usa (ação só com ícone, sempre com `label`).

- **Carregamento**: toda rota que busca dado tem `loading.tsx` com os esqueletos na forma dela, e a parte lenta fica em `<Suspense>` com o esqueleto do mesmo tamanho, nunca `fallback={null}` onde vai aparecer conteúdo (sem isso o clique parece não ter funcionado, e a página pula).
- **Pending**: botão que chama a API recebe `pending` (de `useApiAction` ou `useActionState`); `Button`, `ConfirmButton`, `FormActions`, `OverflowActions` e `UserPanel` já tratam (ignora o clique duplo e o indicador mostra o andamento, sem tirar o foco).
- **Link**: item de lista que abre página é a linha do `DataTable` com `href` ou o `LinkTile`; link solto é o `Link` (azul) e, no meio do texto, `inline` (sublinhado). Sem hover no celular, cor e sublinhado mostram o que é clicável.
- **Hierarquia**: título de página no `PageHeader` (`heading-04`), de seção na `Section` (`heading-03`), de bloco `heading-02`; corpo `body-01`; metadado e dica `helper-text-01` em `$text-secondary`.
- **Contraste**: só os tokens de cor do tema, que dão AA (4.5:1) em `g10` e em `g100`; nunca `opacity` em texto. Confira toda tela nos dois temas.
- **Toque**: abaixo de `md`, controles com 48px (regra global dos tokens; `Link` solto e `LinkTile` também).
- **Foco**: os componentes do Carbon têm o contorno; os controles nativos ganham o do `globals.scss`. Não remova `outline`. Depois de uma ação cujo botão some no refresh, leve o foco para o item afetado (`focusIfLost` de `shared/lib/focus.ts`, com `tabIndex={-1}` no alvo); o Modal e o painel do cabeçalho já devolvem o foco. Campo recusado dentro de acordeão ou aba fechada: abra o que o esconde e use `focusWhenShown`; a validação nativa não abre nada sozinha.
- **Ação destrutiva ou que não volta**: `ConfirmButton`, ou item com `confirm` no `OverflowActions` (o Modal diz o que acontece; o primário tem o nome da ação; "Não" recebe o foco). Grave e raro: `confirmSlug`. Nada de `window.confirm`.
- **Celular (390px)**: sem rolagem horizontal fora da chave (o `DataTable` esconde as colunas `secondary`, o `PageHeader` desce as ações, o `TileGrid` vira uma coluna, o `ProgressSteps` fica vertical (navegável e com mais de 4 passos, um `Dropdown`), títulos longos quebram). O uso principal é no computador, mas parte dos jogadores abre o link pelo celular.
- **Formulários: validação** (o campo cobra o limite; a descrição não: erro depois do envio faz digitar duas vezes, e limite na descrição ninguém lê):
  - Obrigatório: `required` e a estrela vermelha no rótulo (`TextInput`, `PasswordInput`, `TextArea`, `Select`, `NumberInput` e `DateTimeInput` a põem sozinhos; `RequiredLabel` para campo montado à mão), com "obrigatório" para o leitor de tela. Opcional não leva marca.
  - Envio: `useFormValidity()` (`ref` no `<Form>`) dá `canSubmit`, e o botão fica `disabled={!canSubmit}` enquanto faltar obrigatório ou houver campo inválido (desabilitado, e não habilitado com foco no primeiro inválido: o pedido é prevenir o envio, e a causa está à vista).
  - Caractere que o campo não aceita não entra: `filter` do `TextInput` (`slugChars`, `digitsOnly` em `shared/lib/input-filters.ts`); `maxLength` corta o excesso; `NumberInput` recusa letra, `e`, `+`, vírgula e ponto (e `-` sem `negative`).
  - Valor fora do limite fica inválido com a mensagem curta no `invalidText`: número, na hora (`min`/`max`, "Use de 1 a 10."); texto (`minLength`, `pattern` com `patternMessage`, `type="email"`, `required`, `sameAs` da senha repetida), depois da primeira saída do campo e dali a cada tecla ("mínimo de 3" não fica vermelho na primeira letra).
  - Regra que só o chamador sabe (data à frente de agora): `problem` do `TextInput`/`DateTimeInput`, com a mensagem traduzida; aparece na hora e bloqueia o envio.
  - `helperText` só existe quando o rótulo não basta: diz o que o campo é ou um efeito que não se vê, nunca a faixa nem o tamanho. O erro da API continua no `invalid`/`invalidText` (`invalidProps`) e vale mais que o do campo.
- **Estado vazio e erro**: o `EmptyState` diz o estado e põe a ação, sem ensinar o uso ("Como escrever", em `messages/CLAUDE.md`); erro sempre traduzido, junto do controle que falhou (`invalidText`, `ErrorText`) ou acima dos botões (`FormError`).

## MCP do Carbon

- Antes do MCP, o que o projeto já tem: o componente da `shared/ui` (o catálogo acima e o JSDoc dele), um uso dele em `features/` e o `.d.ts` instalado. Cada `docs_search` traz uns 10 mil caracteres: use-o para o padrão que o projeto ainda não usa e, decidido o padrão, registre-o no JSDoc do componente (ou em `docs/design.md`, se for de página), para a próxima tela não buscar de novo.
- `code_audit` nos arquivos que você criou ou mudou, em lote. Ignore quando ele sugerir `<Theme>` em volta da página ou classes `cds--type-*`: o tema vem do `globals.scss`, o tipo do mixin `type-style`.
- A versão instalada manda: na dúvida sobre uma prop, confira o `.d.ts` em `node_modules/@carbon/react/lib/components/<Componente>/`.
