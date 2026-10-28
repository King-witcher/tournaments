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
    lib/                funções puras do domínio (ex.: slugify)
  shared/               o que serve a qualquer domínio; nunca importa de features/
    ui/                 a biblioteca sobre o Carbon (inventário em "Biblioteca shared/ui")
    hooks/              hooks genéricos (ex.: useCopyToClipboard)
    api/                client, serverApi, callApi, useApiAction, tradução de erro, schema.d.ts
    lib/                utilitários genéricos (cn, focus, datetime-local)
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

## Estilo (Carbon)

### Como está montado

- `src/app/globals.scss` (o primeiro import do `layout.tsx`): os estilos do Carbon **por componente** (o `@use "@carbon/react"` inteiro dava 1,25 MB de CSS; a lista dá ~0,6 MB), o tema `white` em `:root` e o `g100` em `prefers-color-scheme: dark`, a regra de toque e o `:focus-visible` dos controles nativos. **Componente do Carbon que não está na lista aparece sem estilo**: acrescente `@use "@carbon/react/scss/components/<pasta>";` (o nome da pasta em `node_modules/@carbon/styles/scss/components`), em ordem alfabética, sem mexer no resto do arquivo.
- Fonte: IBM Plex Sans e Mono do `next/font/google` (sem CDN), nas variáveis `--font-plex-sans` e `--font-plex-mono` do `<html>`. O `$font-families` do Carbon lê essas variáveis, e os tokens de tipo saem como `--cds-*` (por isso `type.type-style("code-02")` num módulo já pega a Plex Mono).
- Toque: abaixo de `md` (672px), os tamanhos `sm` e `md` dos controles do Carbon (botão, campo, select, ⋯, abas, acordeão, paginação, busca) valem `lg` (48px), por token em `globals.scss`. Nenhum componente precisa tratar: passe o tamanho de desktop (`md`, o padrão; `sm` em linha de tabela).
- O `.scss` passa por um `sass-loader` próprio (`turbopack.rules` no `next.config.ts`): o embutido do Turbopack não acha os `@use` relativos do Carbon. Não volte para o embutido.

### SCSS module com tokens

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

- Espaço: `$spacing-01` 2px, `02` 4, `03` 8, `04` 12, `05` 16, `06` 24, `07` 32, `08` 40, `09` 48, `10` 64, `11` 80, `12` 96, `13` 160.
- Cor: `$background`, `$layer-01`, `$layer-hover-01`, `$layer-accent-01`, `$field`, `$border-subtle-01`, `$border-strong`, `$text-primary`, `$text-secondary`, `$text-helper`, `$text-placeholder`, `$text-error`, `$text-disabled`, `$link-primary`, `$link-primary-hover`, `$support-error` (e `-warning`, `-success`, `-info`), `$focus`, `$interactive`.
- Tipo (`type.type-style(...)`): `heading-04` título de página, `heading-03` de seção, `heading-02`/`heading-compact-02` de bloco, `body-01`/`body-02` texto, `body-compact-01` linha de lista, `label-01` rótulo, `helper-text-01` dica e metadado, `code-02` código. Peso: `@include type.font-weight("semibold");`.
- Pontos de quebra: `sm` 320, `md` 672, `lg` 1056, `xlg` 1312, `max` 1584. `breakpoint(md)` é "do md para cima"; `breakpoint-down(md)` é "abaixo do md" (o celular).
- Utilitários: `@use "@carbon/react/scss/utilities/visually-hidden" as *;` com `@include visually-hidden;` (texto só para leitor de tela); `@use "@carbon/react/scss/utilities/skeleton" as *;` com `@include skeleton;`; `@use "@carbon/react/scss/utilities/z-index";` com `z-index.z("modal")`.
- Para mudar uma propriedade que o próprio Carbon põe no elemento dele (o padding do `Content`, o `display` do `ProgressIndicator`), ponha um elemento seu em volta ou dobre a classe (`.panel.panel { padding: 0; }`) para vencer a do Carbon, que tem o mesmo peso.

### O que não fazer

- `style={{…}}`; px, rem ou cor soltos (a única medida fora de token é a borda de `1px`); `!important`.
- Classe `cds--…` escrita à mão ou `:global(.cds--…)`: use o componente, uma prop dele ou um módulo seu.
- Tailwind (saiu: `text-muted`, `min-h-11`, `flex gap-2` e afins não fazem mais nada), `@carbon/labs` e `@carbon/ibm-products`.
- `next/link`, `Link` do Carbon com `href` cru, `<a>` para página interna: use `Link`/`ButtonLink` de `shared/ui`.
- Texto em inglês que o Carbon põe por padrão (aria-label "Close", "Clear search input", "Show password"): os componentes de `shared/ui` já traduzem; usando um do Carbon direto, passe a prop de texto traduzida.

## Biblioteca `shared/ui`

Cada componente já cumpre os princípios (foco, `pending`, toque, celular, confirmação): use-os em vez de montar com o Carbon cru. Os de layout e estrutura sem regra própria (`Grid`, `Column`, `Stack`, `Layer`, `Tile`, `Form`, `FormGroup`, `ButtonSet`, `SkeletonText`, `SkeletonPlaceholder`, `InlineLoading`, `AccordionItem`, `SelectItem`, `UnorderedList`/`ListItem`, `CodeSnippet`) podem vir direto de `@carbon/react`, também em Server Component (o índice do pacote é `"use client"` e serve dos dois lados enquanto as props forem serializáveis). Os marcados com (c) são Client Components; os outros servem em Server e em Client Component.

### Casca e página

- **`AppHeader`** (c), **`AppContent`**, **`HeaderPanelAction`** (c), **`HeaderPlaceholder`**, **`LocaleSwitcher`** (c), **`UserPanel`** (c): a casca, montada em `app/[locale]/layout.tsx`. `UserPanel` recebe `label`, `name`, `avatarSrc` e `items` (`{ label, href }` ou `{ label, onSelect, pending }`); os itens vêm de `features/auth` (`UserMenu`). Os fluxos não mexem aqui.
- **`Page`** `width="narrow" | "wide"`: a coluna da página (`Grid`/`Column`) e o espaço entre os blocos (`Stack gap={7}`). `narrow` (padrão): texto e formulário, 8 colunas centradas a partir de `lg`. `wide`: tabela, grade de tiles, chave.

  ```tsx
  <Page width="wide">
    <PageHeader title={t("title")} />
    <Suspense fallback={<TableSkeleton />}>
      <MemberList slug={slug} />
    </Suspense>
  </Page>
  ```
- **`PageHeader`**: o topo de toda página. `breadcrumbs` (`{ href, label }[]`, da raiz até a mãe; a atual é o título), `title`, `tags` (`StatusTag`s), `meta` (uma linha: formato · início · inscritos), `actions` (uma `primary` e as secundárias, na ordem), `overflow` (um `OverflowActions` com o raro e o destrutivo). No celular as ações descem para baixo do título.

  ```tsx
  <PageHeader
    breadcrumbs={[{ href: "/", label: tNav("home") }, { href: `/${tenant}`, label: tenantName }]}
    title={name}
    tags={<StatusTag status={tournament.status}>{tStatus(tournament.status)}</StatusTag>}
    meta={`${format} · ${t("participants", { count, max })}`}
    actions={<RegistrationButton tenant={tenant} slug={slug} />}
    overflow={<TournamentOverflow tenant={tenant} slug={slug} />}
  />
  ```
- **`PageTabs`** (c) `tabs={{ href, label }[]} label children`: abas do Carbon que são links para sub-rotas; a atual sai do endereço (href igual ou o prefixo mais longo). Vai no `layout.tsx` do segmento, com as sub-rotas em `children` (cada uma com o próprio `loading.tsx`). Aba que não se aplica (Regras sem regras, Organização para jogador) simplesmente não entra na lista.

  ```tsx
  // app/[locale]/[tenant]/layout.tsx
  <Page width="wide">
    <TenantHeader slug={tenant} />
    <PageTabs label={t("sections")} tabs={[
      { href: `/${tenant}`, label: t("tabs.tournaments") },
      { href: `/${tenant}/members`, label: t("tabs.members") },
    ]}>
      {children}
    </PageTabs>
  </Page>
  ```
- **`Section`** `title description? action? id?`: uma seção (h2 `heading-03`, descrição, ação à direita do título; no celular, embaixo) e o conteúdo. Uma coisa por seção.
- **`EmptyState`** `action?`: lista sem nada. O texto diz o que falta e o próximo passo; a ação resolve. `<EmptyState action={<ButtonLink href="/new">{t("create")}</ButtonLink>}>{t("empty")}</EmptyState>`.
- **`PageError`** `href label`: erro que ocupa a página (não encontrado, sem acesso): o `Notice` de erro e o link de volta.

### Ação

- **`Button`** (c): `kind` (`primary`, só um por área; `secondary`, `tertiary`, `ghost`, `danger`, `danger--tertiary`, `danger--ghost`), `size` (`md` padrão, `sm` em linha de tabela, `lg`), `pending`, `renderIcon` (ícone do `@carbon/icons-react`, só em Client Component), `type` (`button` padrão). `pending` não usa `disabled`: fica com `aria-disabled`/`aria-busy`, ignora clique e Enter e mostra o `InlineLoading` dentro, sem perder o foco.

  ```tsx
  const { run, pending, error } = useApiAction();
  <Button pending={pending} onClick={() => run(() => api.POST(path), () => router.refresh())}>
    {t("checkIn")}
  </Button>
  {error && <ErrorText>{error}</ErrorText>}
  ```
- **`ButtonLink`** (c) `href kind size renderIcon`: link com cara de botão, para ação que leva a outra página (Criar, Editar, Entrar). `<ButtonLink href={`/${slug}/new`} kind="tertiary">{t("create")}</ButtonLink>`.
- **`Link`** (c) `href inline size renderIcon`: o link do Carbon sobre o do i18n. Solto: azul e, no celular, 48px de altura. No meio de um parágrafo: `inline` (sublinhado). `<Link href={`/${tenant}`}>{t("backToCommunity")}</Link>`.
- **`ConfirmButton`** (c): ação que não se desfaz fácil. O clique abre o `Modal` com `title` (padrão: o texto do botão), `question` (o que vai acontecer), o primário com o mesmo texto do botão e "Não", que recebe o foco; ao fechar, o foco volta ao botão, que fica `pending` enquanto a ação roda. `destructive` (padrão `true`): Modal `danger`; `false` para o que só avança (publicar, iniciar). `confirmSlug`: o primário só libera com o slug digitado (desfazer início, transferir posse). Padrões: `size="sm"` e `kind` `danger--tertiary` (destrutivo) ou `tertiary`.

  ```tsx
  <ConfirmButton
    question={t("removeConfirm", { name })}
    pending={pending}
    onConfirm={() => run(() => api.DELETE(path), () => router.refresh())}
  >
    {t("remove")}
  </ConfirmButton>
  ```
- **`ConfirmModal`** (c) `open onClose onConfirm title confirmLabel destructive slug`: o mesmo Modal, para quem o abre por conta (um formulário que confirma antes de enviar). Quem abre devolve o foco ao fechar.
- **`OverflowActions`** (c) `items label? pending? size?`: o menu ⋯ (por linha de tabela ou no `PageHeader`). Item: `{ label, onSelect?, href?, destructive?, disabled?, confirm?: { question, title?, slug? } }`; com `confirm`, abre o `ConfirmModal` e devolve o foco ao ⋯. `pending` mostra o indicador ao lado.

  ```tsx
  <OverflowActions pending={pending} items={[
    { label: t("makeOrganizer"), onSelect: () => run(promote, refresh) },
    { label: t("remove"), destructive: true,
      confirm: { question: t("removeConfirm", { name }) },
      onSelect: () => run(remove, refresh) },
  ]} />
  ```

### Formulário

O padrão: `Form` (do `@carbon/react`, com `action={submit}` do `useActionState`), um `FormGroup legendText` por seção, `Stack gap={6}` dentro, campos com `labelText`, `helperText`, `invalid` e `invalidText`, `FormError` acima dos botões e `FormActions` no fim. Campo travado: `Notice kind="info"` com o motivo **antes** dos campos.

```tsx
<Form action={submit}>
  <Stack gap={7}>
    <FormGroup legendText={t("identity")}>
      <Stack gap={6}>
        <TextInput
          labelText={t("name")}
          helperText={t("nameHint")}
          name="name"
          required
          invalid={nameError !== undefined}
          invalidText={nameError}
        />
        <Select
          labelText={t("visibility")}
          value={visibility}
          onChange={(e) => setVisibility(e.target.value)}
        >
          <SelectItem value="public" text={t("public")} />
        </Select>
      </Stack>
    </FormGroup>
    {error && <FormError>{error}</FormError>}
    <FormActions pending={pending} cancelHref={`/${tenant}`} />
  </Stack>
</Form>;
```

- **`TextInput`**, **`PasswordInput`**, **`TextArea`**, **`Select`** (c): os do Carbon com `id` gerado (e, no `PasswordInput`, o mostrar/esconder traduzido); as props são as do Carbon. `Select` aceita `<option>` ou `SelectItem`.
- **`NumberInput`** (c) `labelText value onChange min max step`: `value` e `onChange` em `number | null` (vazio é `null`, com `allowEmpty`).
- **`DateTimeInput`** (c) `labelText value onChange helperText`: data e hora no fuso de quem usa; entra e sai em ISO (UTC); vazio até hidratar.
- **`Checkbox`** (c) `labelText checked onChange(boolean)`; **`RadioGroup`** (c) `legendText options={{ value, label }[]} value onChange orientation` (até ~5 opções; mais que isso, `Select`).
- **`FormError`** `title?`: o erro geral (o que a API devolveu), em `InlineNotification` de erro. Erro de um campo vai no `invalidText` dele.
- **`FormActions`** (c) `submitLabel? pending disabled cancelHref? onCancel? cancelLabel? destructive?`: o `ButtonSet` com "Cancelar" secundário à esquerda e o primário (`type="submit"`, padrão "Salvar") à direita.

### Listas

- **`DataTable`** (c): lista grande (inscritos, membros, convites, torneios da comunidade), com busca na barra, paginação no cliente (25, 50 ou 100; aparece com mais de 25 linhas), linha que abre página, ações por linha e estado vazio. As células chegam prontas (o Server Component monta as linhas) e a busca procura em `search`. `columns`: `{ key, header, secondary? }` (`secondary` some até `md`; deixe sem a principal e a de estado). `rows`: `{ id, cells, search?, href?, actions? }`; com `href`, a célula principal (`primaryColumn`, padrão a primeira) vira link e o clique na linha leva ao mesmo lugar. `toolbar`: a ação da barra (a primária da aba). `empty`: `{ text, action? }`.

  ```tsx
  <DataTable
    label={t("title")}
    columns={[
      { key: "name", header: t("member") },
      { key: "role", header: t("role") },
      { key: "since", header: t("since"), secondary: true },
    ]}
    rows={members.map((m) => ({
      id: m.slug,
      search: m.nickname,
      cells: {
        name: <MemberName member={m} />,
        role: <StatusTag status="neutral">{tRoles(m.role)}</StatusTag>,
        since: <LocalDateTime value={m.joined_at} />,
      },
      actions: canManage ? <MemberActions member={m} /> : undefined,
    }))}
    empty={{ text: t("empty") }}
  />
  ```
- **`TileGrid`** + **`LinkTile`** (c): as listas "minhas" (seus torneios, suas comunidades). `TileGrid` é a grade (1, 2 ou 3 colunas); `LinkTile` `href title meta? tags? footer?` é o `ClickableTile` inteiro como link, com um link à parte embaixo em `footer` (a próxima partida). Exemplo completo: `features/tournaments/components/my-tournament-tile.tsx`.
- **`StructuredList`** `label headers? rows={{ key, cells }[]} condensed?`: só leitura, em colunas: o resumo do formato (sem `headers`: rótulo e valor) e os mapas da partida.

### Estado e retorno

- **`StatusTag`** `status size?`: a `Tag` com a cor do significado. `draft` cinza, `registration` azul, `check_in` roxo, `in_progress` verde, `finished` cinza frio, `cancelled` vermelho (cancelado, desistiu, saiu, perdeu a vaga), `waiting` verde-azulado, `contested` magenta, `confirmed` verde, `neutral` cinza (papel, contagem). Os códigos do torneio são os da API: `<StatusTag status={tournament.status}>{tStatus(tournament.status)}</StatusTag>`. Código da API com outro nome: um mapa `Record<CódigoDaApi, Status>` na feature (ver `my-tournament-tile.tsx`).
- **`Notice`** `kind="error" | "warning" | "info" | "success"` `title?`: aviso na página (`InlineNotification` sem fechar); o de erro é `role="alert"`.
- **`ErrorText`** (em `@/shared/ui/alert`): erro curto junto do botão de linha que falhou (o `error` do `useApiAction`).
- **`Toast`** (c) `open onClose title kind? timeout?`: só para "copiado" e afins; some sozinho.
- **`ProgressSteps`** (c) `steps={{ key, label, secondaryLabel? }[]} current label`: o ciclo do torneio, só leitura; vertical no celular.
- **`Accordion`** `title open?`: um bloco fechado (encerrados, histórico, organização da partida). Vários seguidos: `Accordion` e `AccordionItem` do Carbon direto.
- **Esqueletos** (`@/shared/ui/skeleton`): `PageHeaderSkeleton breadcrumb? meta? label?` (leva o "Carregando…" para leitor de tela; um por página), `TabsSkeleton`, `SectionSkeleton tiles? rows?`, `TableSkeleton columns? rows?`, `TileGridSkeleton count?`, `LoadingSection label? rows?` (com o texto visível) e `PageSkeleton` (genérico). Monte o `loading.tsx` na forma da página:

  ```tsx
  export default function Loading() {
    return (
      <Page width="wide">
        <PageHeaderSkeleton meta />
        <TabsSkeleton />
        <TableSkeleton columns={4} />
      </Page>
    );
  }
  ```

### Conteúdo

- **`Avatar`** `src? name? size="sm" | "md" | "lg"` (24, 32 e 48px): sem imagem, a inicial. Decorativo: o nome vai sempre ao lado.
- **`Markdown`**: texto de usuário (descrição, regras) com a tipografia do Carbon, em coluna de leitura.
- **`Countdown`** (c) `until onExpire?`, **`LocalDateTime`** (c) `value`, **`LocalTime`** (c) `value`: tempo e data no fuso de quem vê (UTC até hidratar).

### Antigos (adaptadores `@deprecated`, só até o fluxo refazer a tela)

`Field` + `Input`/`Textarea` → `TextInput`, `TextArea`, `Select` com `labelText`; `Alert` → `Notice kind="error"` ou `FormError`; `Badge` → `StatusTag`; `RowLink` → `DataTable` com `href` ou `TileGrid` + `LinkTile`; `Disclosure` → `Accordion`; `Skeleton` → as peças de esqueleto; `Button variant` → `kind` (`default`→`primary`, `outline`→`tertiary`, `destructive`→`danger`, `link`→botão com cara de link); `ConfirmButton danger`/`variant` → `destructive`/`kind`; `Page spacing` → nada. Tela refeita não usa nenhum deles.

## Textos (messages)

- Um arquivo por namespace de primeiro nível: `messages/pt/Tenant.json` e `messages/es/Tenant.json`. Chave nova vai no namespace do seu fluxo, nos dois idiomas. `es` precisa ter as mesmas chaves de `pt`: se faltar, o `tsc` e o `build` acusam.
- Namespace novo: crie os dois arquivos e acrescente uma linha em cada lista (`pt` e `es`) de `src/i18n/namespaces.ts`, em ordem alfabética.
- `Common` (os textos da biblioteca: carregando, fechar, cancelar, salvar, tabela, avisos) e `Nav` são da base. Num namespace que outro fluxo também usa, só acrescente chaves: não reordene nem reformate o arquivo.
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
- **Foco**: os componentes do Carbon têm o contorno de foco; os controles nativos ganham o do `globals.scss`. Não remova `outline`. Depois de uma ação cujo botão some no refresh, leve o foco para o item afetado (`focusIfLost` de `shared/lib/focus.ts`, com `tabIndex={-1}` no alvo); o Modal e o painel do cabeçalho já devolvem o foco. Motivo: quem usa teclado precisa ver onde está e não voltar ao começo da página a cada ação.
- **Ação destrutiva ou que não volta**: `ConfirmButton`, ou item com `confirm` no `OverflowActions` (o Modal diz o que acontece; o primário tem o nome da ação; "Não" recebe o foco). Grave e raro: `confirmSlug`. Nada de `window.confirm`.
- **Celular (390px)**: nada de rolagem horizontal fora da chave: o `DataTable` esconde as colunas `secondary`, o `PageHeader` desce as ações, o `TileGrid` vira uma coluna, o `ProgressSteps` fica vertical, títulos longos quebram. Motivo: a maior parte dos jogadores abre o link do torneio pelo celular.
- **Estado vazio e erro**: o `EmptyState` diz o próximo passo, com a ação; erro sempre traduzido, junto do controle que falhou (`invalidText`, `ErrorText`) ou acima dos botões (`FormError`).

## MCP do Carbon

Três ferramentas (`mcp__ibm-carbon-design__*`); a resposta vem em JSON como texto.

- `code_search`: props, variantes e exemplos de um componente ou o nome de um ícone. Sempre com `filters: { component_type: "React" }` e, quando souber, `component_id` (`"data-table"`, `"modal"`, `"tabs"`…), `size` 1 ou 2 e o nome exato do componente na query. Ícone: a query com o nome, sem `component_type`.
- `docs_search`: uso, acessibilidade, conteúdo e tokens ("data table empty state", "inline notification accessibility", "spacing tokens"), com `size: 3`.
- `code_audit`: valida arquivos contra as regras do Carbon (tokens, componentes, acessibilidade). Rode nos arquivos que você criou ou mudou, em lote (`files: [{ path, code }]`, até 50). Ele sugere `<Theme>` em volta da página e classes `cds--type-*`: aqui não (o tema vem do `globals.scss`; o tipo, do mixin `type-style` no módulo).
- A versão instalada manda: na dúvida sobre uma prop, confira o `.d.ts` em `node_modules/@carbon/react/lib/components/<Componente>/`.
