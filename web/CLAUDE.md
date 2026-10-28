@AGENTS.md

# Frontend (web/)

- Só apresentação: nunca acessa o banco. Os dados vêm da API Rust pelo client `api` em `src/shared/api/client.ts` (openapi-fetch).
- Gerenciador de pacotes e scripts: Bun (`bun add`, `bun run ...`). Lint e formatação: Biome (`bun run lint`, `bun run format`), não ESLint/Prettier. O Biome também confere os `.scss`.
- `src/shared/api/schema.d.ts` é gerado: não edite à mão. Com a API rodando, rode `bun run gen:api`.
- Todo texto visível vem de `messages/<idioma>/<Namespace>.json` (next-intl), um arquivo por namespace. Nada de texto direto no JSX; toda chave nova entra em `pt` e `es` (ver "Textos").
- Códigos que a API devolve (status, erros) são traduzidos aqui.
- O idioma é o segmento raiz `[locale]`. No servidor, `getTranslations` já lê o idioma de `next/root-params`; não passe locale à mão.
- Links e redirects pelo `src/i18n/navigation.ts`, não por `next/link`/`next/navigation`, para manter o prefixo de idioma. Link visível na tela: `Link`/`ButtonLink` de `shared/ui` (já usam o do i18n por baixo). O tipo de um destino é `Href`, de `@/i18n/navigation`.
- Cache Components está ligado: dado buscado a cada requisição fica dentro de `<Suspense>`.
- Interface: Carbon Design System v11 (`@carbon/react`, `@carbon/icons-react`). O que cada página é está em `docs/design.md`; aqui fica o como.

## Organização

Siga ao criar arquivo, componente ou hook.

```
src/
  app/[locale]/...      só rotas: lê params, monta <Page>, <Suspense> e componentes de feature
    (home)/             o início (/), num grupo para ter o próprio loading.tsx
  app/globals.scss      Carbon (lista de componentes), temas, fonte, toque, foco
  features/<domínio>/   tudo de um domínio; os mesmos domínios da API (auth, tenants, members,
                        invites, health, tournaments, participants, lifecycle, matches, ...)
    components/         um componente por arquivo (+ o .module.scss dele, com o mesmo nome)
    server.ts           busca no servidor usada por mais de um componente (ex.: getCurrentUser)
    hooks/              hooks do domínio
    lib/                funções puras do domínio (ex.: participants/lib/situation.ts)
  shared/               o que serve a qualquer domínio; nunca importa de features/
    ui/                 a biblioteca sobre o Carbon (inventário em "Biblioteca shared/ui")
    hooks/              hooks genéricos (ex.: useCopyToClipboard, useFocusInvalid)
    api/                client, serverApi, callApi, useApiAction, tradução de erro, schema.d.ts
    lib/                utilitários genéricos
  i18n/                 next-intl: routing, navigation (Link, Href), request, namespaces
messages/<pt|es>/       um JSON por namespace
```

### Onde cada coisa vai

- Componente que conhece um domínio (chama a API, usa tipo do schema, tem texto próprio) fica em `features/<domínio>/components/`. Em `shared/ui` só entra o que serviria a qualquer domínio.
- Nada de `components/`, `utils/` ou `helpers/` soltos em `src/`. Se não sabe onde pôr, é sinal de que falta nomear o domínio.
- Página (`page.tsx`) não busca dado nem tem lógica: compõe. A busca fica no componente de feature.
- Feature pode usar componente de outra (`TenantView` monta `MemberList` e `InviteSection`), sem ciclo. Se duas features precisam da mesma coisa genérica, ela vai para `shared`.

### Nomes e imports

- Arquivo em kebab-case com o nome do que exporta: `member-row.tsx` → `MemberRow`, `use-api-action.ts` → `useApiAction`. Um componente exportado por arquivo; helper pequeno e privado pode ficar junto. O estilo dele vai em `member-row.module.scss`, ao lado.
- Dentro da mesma feature, import relativo (`./member-row`, `../server`); fora dela, `@/features/...` ou `@/shared/...`.
- Sem `index.ts` reexportando: importe o arquivo. Barrel mistura código de servidor e de cliente e atrapalha o bundle.

### Dados e mutações

- Server Component por padrão. `"use client"` só na folha que precisa de estado ou evento (o botão, não a seção).
- Busca: componente async que busca o próprio dado (`MemberList` busca os membros), dentro de `<Suspense>`. Busca usada em mais de um lugar vai para `features/<domínio>/server.ts`, com `cache()` do React.
- Mutação de botão: `useApiAction` (`run(chamada, aoDarCerto)` com `pending` e `error` já traduzido). Formulário: `useActionState` + `callApi`. Nunca mostre o código cru da API.
- Server Component não passa função para Client Component: `onClick`, `onSelect`, `renderIcon` e `as` só existem dentro de arquivo `"use client"`. Lista do servidor com ação por linha: o servidor monta as linhas e põe em cada uma um componente de cliente da feature (`<MemberActions member={m} />`), que tem o `useApiAction` e o `OverflowActions`.

### Tempo real

- Página que se atualiza sozinha: ponha `<LiveUpdates>` (`features/realtime`) dizendo o que ela mostra (`tenant`, `tournament`, `match`, `me`), no `params.then(...)` da página ou do `layout.tsx` do grupo. Página sem `LiveUpdates` só muda ao navegar. O refresh mantém o `useState` dos componentes de cliente, mas `key` derivada do dado remonta o bloco.
- O que recarrega cada página: `features/realtime/lib/interest.ts`. Conexão, toasts e som: `features/realtime`; Web Push e service worker: `features/push` e `app/sw.js/route.ts`.

## Estilo (Carbon)

- `src/app/globals.scss` importa os estilos do Carbon **por componente** (o pacote inteiro dava 1,25 MB de CSS). **Componente do Carbon que não está na lista aparece sem estilo**: acrescente `@use "@carbon/react/scss/components/<pasta>";` (a pasta em `node_modules/@carbon/styles/scss/components`), em ordem alfabética, sem mexer no resto do arquivo.
- Tema `white` em `:root` e `g100` no escuro (do sistema ou `data-theme="dark"`, a escolha da pessoa). Fonte IBM Plex pelo `next/font`; os tokens de tipo já a usam.
- Toque: abaixo de `md` (672px), os controles `sm` e `md` do Carbon valem 48px por token em `globals.scss`. Passe o tamanho de desktop (`md`, o padrão; `sm` em linha de tabela); nenhum componente trata.
- O `.scss` passa por um `sass-loader` próprio (`turbopack.rules` no `next.config.ts`): o embutido do Turbopack não acha os `@use` relativos do Carbon. Não volte para o embutido.

O pouco de CSS próprio (grade, alinhamento, avatar, placar, chave) vai num `.module.scss` ao lado do componente, só com tokens:

```scss
@use "@carbon/react/scss/spacing"; // spacing.$spacing-01 … $spacing-13
@use "@carbon/react/scss/theme"; // theme.$text-secondary, theme.$layer-01 …
@use "@carbon/react/scss/type"; // @include type.type-style("heading-03");
@use "@carbon/react/scss/breakpoint"; // @include breakpoint.breakpoint-down(md) { … }

.row {
  display: flex;
  gap: spacing.$spacing-05;
  color: theme.$text-secondary;

  @include breakpoint.breakpoint-down(md) {
    flex-direction: column;
  }
}
```

- `breakpoint(md)` é "do md para cima"; `breakpoint-down(md)` é "abaixo do md" (o celular). Utilitários do Carbon: `visually-hidden`, `skeleton`, `z-index.z("modal")` (em `@carbon/react/scss/utilities/`).
- Para mudar uma propriedade que o próprio Carbon põe no elemento dele (o padding do `Content`, o `display` do `ProgressIndicator`), ponha um elemento seu em volta ou dobre a classe (`.panel.panel { padding: 0; }`) para vencer a do Carbon, que tem o mesmo peso.

### O que não fazer

- `style={{…}}`; px, rem ou cor soltos (a única medida fora de token é a borda de `1px`); `!important`.
- Classe `cds--…` escrita à mão ou `:global(.cds--…)`: use o componente, uma prop dele ou um módulo seu.
- Tailwind (saiu: `text-muted`, `min-h-11`, `flex gap-2` e afins não fazem mais nada), `@carbon/labs` e `@carbon/ibm-products`.
- `next/link`, `Link` do Carbon com `href` cru, `<a>` para página interna: use `Link`/`ButtonLink` de `shared/ui`.
- `Modal`/`ComposedModal` do Carbon direto numa feature: use `ConfirmButton`, `ConfirmModal`, `FormModal` ou `PassiveModal`.
- Texto em inglês que o Carbon põe por padrão (aria-label "Close", "Clear search input", "Show password"): os componentes de `shared/ui` já traduzem; usando um do Carbon direto, passe a prop de texto traduzida.

## Biblioteca `shared/ui`

Cada componente já cumpre os princípios abaixo (foco, `pending`, toque, celular, confirmação): use-o em vez de montar com o Carbon cru. Props, exemplos e regras de cada um estão no JSDoc do arquivo (`src/shared/ui/<nome>.tsx`); leia antes de usar. Layout e estrutura sem regra própria (`Grid`, `Column`, `Stack`, `Layer`, `Tile`, `Form`, `FormGroup`, `SkeletonText`, `InlineLoading`, `AccordionItem`, `SelectItem`, `UnorderedList`/`ListItem`, `CodeSnippet`) vêm direto de `@carbon/react`, também em Server Component, enquanto as props forem serializáveis.

- **Casca** (montada em `app/[locale]/layout.tsx`; os fluxos não mexem): `AppHeader`, `AppContent`, `HeaderPanelAction`, `HeaderPlaceholder`, `LocaleSwitcher`, `UserPanel`, `ThemeSwitcher`.
- **Página**: `Page` (`narrow` para texto e formulário, `wide` para tabela e chave), `PageHeader` (o topo de toda página: trilha, título, tags, metadado, ações e ⋯), `PageTabs` (abas que são sub-rotas, no `layout.tsx` do segmento), `Section`, `EmptyState`, `PageError`.
- **Ação**: `Button`, `ButtonLink` (ação que leva a outra página), `IconButton`, `Link`, `ConfirmButton` (ação que não se desfaz fácil), `ConfirmModal`, `FormModal` (formulário curto num Modal), `PassiveModal`, `ActionWithNote` (o motivo de não dar ou o erro, embaixo do botão), `OverflowActions` (o ⋯).
- **Formulário**: `FormSection`, `TextInput`, `PasswordInput`, `TextArea`, `Select`, `NumberInput`, `DateTimeInput`, `Toggle`, `Checkbox`, `RadioGroup`, `FormError`, `FormActions`. Erro por campo: `FormResult`, `invalidProps`, `formError`, `fieldId` (`shared/lib/form-result.ts`) e `useFocusInvalid` (`shared/hooks/`).
- **Listas**: `DataTable` (lista grande, com busca e paginação), `TileGrid` + `LinkTile` (as listas "minhas"), `StructuredList` (só leitura, em colunas).
- **Estado e retorno**: `StatusTag`, `Notice`, `ErrorText`, `Toast` ("copiado"), `ToastStack` (avisos que chegam sozinhos), `ProgressSteps`, `Accordion`, os esqueletos de `skeleton.tsx`.
- **Conteúdo**: `Avatar`, `UserText` (texto de usuário sem formatação, com as quebras de linha), `Countdown`, `LocalDateTime`, `LocalDate`, `LocalTime`.

Formulário: `Form` com `action` do `useActionState`, um `FormSection` por grupo de campos (com um grupo só, os campos direto num `Stack gap={6}`), erro de campo no `invalidText` (`invalidProps`), `FormError` acima dos botões e `FormActions` no fim. Campo travado: `Notice kind="info"` com o motivo **antes** dos campos. Modelo: `features/tenants/components/create-tenant-form.tsx`.

## Textos (messages)

- Um arquivo por namespace de primeiro nível: `messages/pt/Tenant.json` e `messages/es/Tenant.json`. Chave nova vai no namespace do seu fluxo, nos dois idiomas. `es` precisa ter as mesmas chaves de `pt`: se faltar, o `tsc` e o `build` acusam.
- Namespace novo: crie os dois arquivos e acrescente uma linha em cada lista (`pt` e `es`) de `src/i18n/namespaces.ts`, em ordem alfabética.
- `Common` (os textos da biblioteca: carregando, fechar, cancelar, salvar, tabela, avisos) e `Nav` são da base. Num namespace que outro fluxo também usa, só acrescente chaves: não reordene nem reformate o arquivo.
- Código novo de aviso na API: `Notifications.notices.<código>` (`title` e `body`) nos dois idiomas, os mesmos na página e no service worker. Só `{param}` simples (o service worker não entende plural do ICU); o prazo `confirm_by` entra como `{deadline}`.
- O espanhol é em "tú", como o resto.

## Quando quebrar um componente

Quebre quando aparecer qualquer um destes sinais:

- passou de ~100 linhas;
- busca dado e também desenha a lista: separe a lista do item (`MemberList` + `MemberRow`);
- tem mais de uma seção com título: uma seção por componente;
- item de lista com mais de ~10 linhas de JSX: `<XRow>` ou `<XTile>`;
- estado que só uma parte usa (ex.: "copiado"): componente da parte (`InviteLink`);
- lógica com estado que se repete: hook (em `shared/hooks` se for genérica).

## Princípios de interface

Utilidade antes de enfeite: cada estilo precisa de um motivo de uso (hierarquia, affordance, legibilidade, toque, foco, leitura no celular). "Fica bonito" não é motivo. Nada de gradiente, sombra decorativa, ícone enfeitando card ou emoji; ícone só onde o Carbon usa (ação só com ícone, sempre com `label`).

- **Carregamento**: toda rota que busca dado tem `loading.tsx` com os esqueletos na forma dela, e a parte lenta fica em `<Suspense>` com o esqueleto do mesmo tamanho, nunca `fallback={null}` onde vai aparecer conteúdo. Motivo: sem isso, o clique parece não ter funcionado, e o esqueleto com a altura do conteúdo evita que a página pule.
- **Pending**: botão que chama a API recebe `pending` (de `useApiAction` ou `useActionState`); `Button`, `ConfirmButton`, `FormActions`, `OverflowActions` e `UserPanel` já tratam. Motivo: ignora o clique duplo e o indicador diz que a ação está em andamento, sem tirar o foco.
- **Link**: item de lista que abre página é a linha do `DataTable` com `href` ou o `LinkTile`; link solto é o `Link` (azul) e, no meio do texto, `inline` (sublinhado). Motivo: sem hover (celular), a cor e o sublinhado mostram o que é clicável.
- **Hierarquia**: título de página no `PageHeader` (`heading-04`), de seção na `Section` (`heading-03`), de bloco `heading-02`; corpo `body-01`; metadado e dica `helper-text-01` em `$text-secondary`. Motivo: título, descrição e metadado se distinguem por tamanho e peso, não só por cor.
- **Contraste**: só os tokens de cor do tema (`$text-secondary`, `$text-helper`…), que dão AA (4.5:1) em `white` e em `g100`; nunca `opacity` em texto. Confira toda tela nos dois temas.
- **Toque**: abaixo de `md`, controles com 48px (a regra global dos tokens; `Link` solto e `LinkTile` também). Motivo: alvo de dedo no celular sem inflar a tela com mouse.
- **Foco**: os componentes do Carbon têm o contorno de foco; os controles nativos ganham o do `globals.scss`. Não remova `outline`. Depois de uma ação cujo botão some no refresh, leve o foco para o item afetado (`focusIfLost` de `shared/lib/focus.ts`, com `tabIndex={-1}` no alvo); o Modal e o painel do cabeçalho já devolvem o foco. Campo recusado dentro de acordeão ou aba fechada: abra o que o esconde e use `focusWhenShown` (espera o campo aparecer); a validação nativa não abre nada sozinha, por isso o formulário de torneio usa `noValidate` e confere antes de enviar. Motivo: quem usa teclado precisa ver onde está e não voltar ao começo da página a cada ação.
- **Ação destrutiva ou que não volta**: `ConfirmButton`, ou item com `confirm` no `OverflowActions` (o Modal diz o que acontece; o primário tem o nome da ação; "Não" recebe o foco). Grave e raro: `confirmSlug`. Nada de `window.confirm`.
- **Celular (390px)**: nada de rolagem horizontal fora da chave: o `DataTable` esconde as colunas `secondary`, o `PageHeader` desce as ações, o `TileGrid` vira uma coluna, o `ProgressSteps` fica vertical, títulos longos quebram. Motivo: o uso principal é no computador, mas parte dos jogadores abre o link do torneio pelo celular.
- **Estado vazio e erro**: o `EmptyState` diz o próximo passo, com a ação; erro sempre traduzido, junto do controle que falhou (`invalidText`, `ErrorText`) ou acima dos botões (`FormError`).

## MCP do Carbon

- `code_audit` nos arquivos que você criou ou mudou, em lote. Ignore quando ele sugerir `<Theme>` em volta da página ou classes `cds--type-*`: o tema vem do `globals.scss`; o tipo, do mixin `type-style` no módulo.
- A versão instalada manda: na dúvida sobre uma prop, confira o `.d.ts` em `node_modules/@carbon/react/lib/components/<Componente>/`.
