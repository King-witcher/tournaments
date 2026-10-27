@AGENTS.md

# Frontend (web/)

- Só apresentação: nunca acessa o banco. Os dados vêm da API Rust pelo client `api` em `src/shared/api/client.ts` (openapi-fetch).
- Gerenciador de pacotes e scripts: Bun (`bun add`, `bun run ...`). Lint e formatação: Biome (`bun run lint`, `bun run format`), não ESLint/Prettier.
- `src/shared/api/schema.d.ts` é gerado: não edite à mão. Com a API rodando, rode `bun run gen:api`.
- Todo texto visível vem de `messages/pt.json` e `messages/es.json` (next-intl). Nada de texto direto no JSX; toda chave nova entra nos dois arquivos.
- Códigos que a API devolve (status, erros) são traduzidos aqui.
- O idioma é o segmento raiz `[locale]`. No servidor, `getTranslations` já lê o idioma de `next/root-params`; não passe locale à mão.
- Links e redirects pelo `src/i18n/navigation.ts`, não por `next/link`/`next/navigation`, para manter o prefixo de idioma.
- Cache Components está ligado: dado buscado a cada requisição fica dentro de `<Suspense>`.

## Organização

Siga ao criar arquivo, componente ou hook.

```
src/
  app/[locale]/...      só rotas: lê params, monta <Page>, <Suspense> e componentes de feature
  features/<domínio>/   tudo de um domínio; os mesmos domínios da API (auth, tenants, members,
                        invites, health; depois tournaments, participants, matches, veto, ...)
    components/         um componente por arquivo
    server.ts           busca no servidor usada por mais de um componente (ex.: getCurrentUser)
    hooks/              hooks do domínio
    lib/                funções puras do domínio (ex.: slugify)
  shared/               o que serve a qualquer domínio; nunca importa de features/
    ui/                 componentes visuais genéricos: Button, Input, Field, Alert, Avatar, Page
    hooks/              hooks genéricos (ex.: useCopyToClipboard)
    api/                client, serverApi, callApi, useApiAction, tradução de erro, schema.d.ts
    lib/                utilitários genéricos (ex.: cn)
  i18n/                 configuração do next-intl
```

### Onde cada coisa vai

- Componente que conhece um domínio (chama a API, usa tipo do schema, tem texto próprio) fica em `features/<domínio>/components/`. Em `shared/ui` só entra o que serviria a qualquer domínio.
- Nada de `components/`, `utils/` ou `helpers/` soltos em `src/`. Se não sabe onde pôr, é sinal de que falta nomear o domínio.
- Página (`page.tsx`) não busca dado nem tem lógica: compõe. A busca fica no componente de feature.
- Feature pode usar componente de outra (`TenantView` monta `MemberList` e `InviteSection`), sem ciclo. Se duas features precisam da mesma coisa genérica, ela vai para `shared`.

### Nomes e imports

- Arquivo em kebab-case com o nome do que exporta: `member-row.tsx` → `MemberRow`, `use-api-action.ts` → `useApiAction`. Um componente exportado por arquivo; helper pequeno e privado pode ficar junto.
- Dentro da mesma feature, import relativo (`./member-row`, `../server`); fora dela, `@/features/...` ou `@/shared/...`.
- Sem `index.ts` reexportando: importe o arquivo. Barrel mistura código de servidor e de cliente e atrapalha o bundle.

### Dados e mutações

- Server Component por padrão. `"use client"` só na folha que precisa de estado ou evento (o botão, não a seção).
- Busca: componente async que busca o próprio dado (`MemberList` busca os membros), dentro de `<Suspense>`. Busca usada em mais de um lugar vai para `features/<domínio>/server.ts`, com `cache()` do React.
- Mutação de botão: `useApiAction` (`run(chamada, aoDarCerto)` com `pending` e `error` já traduzido). Formulário: `useActionState` + `callApi`. Nunca mostre o código cru da API.

### Estilo

- Use `shared/ui` em vez de repetir classes: `Button` com `variant` (`default`, `outline`, `link`) e `size` (`default`, `sm`); `buttonVariants()` para `<Link>` com cara de botão; `Field` + `Input` em formulário; `Alert` (erro principal) e `ErrorText` (erro junto do controle).
- Não há tailwind-merge: `className` extra só para o que a variante não define (largura, cor, fonte). Precisa mudar padding ou fundo? Crie uma variante.
- shadcn, quando entrar: `components.json` com `aliases.ui = "@/shared/ui"`, `aliases.utils = "@/shared/lib/cn"` e `aliases.hooks = "@/shared/hooks"`. Os primitivos atuais já usam os nomes de variante dele.

### Quando quebrar um componente

Quebre quando aparecer qualquer um destes sinais:

- passou de ~100 linhas;
- busca dado e também desenha a lista: separe a lista do item (`MemberList` + `MemberRow`);
- tem mais de uma seção com título: uma seção por componente;
- item de lista com mais de ~10 linhas de JSX: `<XRow>`;
- estado que só uma parte usa (ex.: "copiado"): componente da parte (`InviteLink`);
- lógica com estado que se repete: hook (em `shared/hooks` se for genérica).
