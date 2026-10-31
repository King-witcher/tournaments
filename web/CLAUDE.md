@AGENTS.md

# Frontend (web/)

- Só apresentação: nunca acessa o banco. Os dados vêm da API Rust pelo client `api` (`src/shared/api/client.ts`, openapi-fetch).
- Bun para pacotes e scripts; Biome para lint e formatação (não ESLint/Prettier; confere também os `.scss`). `src/shared/api/schema.d.ts` é gerado: não edite à mão, rode `bun run gen:api`.
- Todo texto visível vem de `messages/<idioma>/<Namespace>.json` (next-intl), nada de texto no JSX; chave nova entra em `pt` e `es`. Códigos da API (status, erros) são traduzidos aqui.
- O idioma é o segmento raiz `[locale]`; no servidor, `getTranslations` já lê o idioma de `next/root-params` (não passe locale à mão).
- Links e redirects pelo `src/i18n/navigation.ts`, não por `next/link`/`next/navigation`, para manter o prefixo de idioma. Link visível: `Link`/`ButtonLink` de `shared/ui`. Tipo de destino: `Href`, de `@/i18n/navigation`.
- Cache Components está ligado: dado buscado a cada requisição fica dentro de `<Suspense>`.
- Interface: Carbon Design System v11 (`@carbon/react`, `@carbon/icons-react`). O que cada página é está em `docs/design.md`; aqui fica o como.

## Organização

```
src/
  app/[locale]/...      só rotas: lê params, monta <Page>, <Suspense> e componentes de feature
    (home)/             o início (/), num grupo para ter o próprio loading.tsx
  app/globals.scss      Carbon (lista de componentes), temas, fonte, toque, foco
  features/<domínio>/   tudo de um domínio, os mesmos da API
    components/         um componente por arquivo (+ o .module.scss com o mesmo nome)
    server.ts           busca no servidor usada por mais de um componente
    hooks/  lib/        hooks e funções puras do domínio
  shared/               serve a qualquer domínio; nunca importa de features/
    ui/  hooks/  api/  lib/
  i18n/                 next-intl: routing, navigation, request, namespaces
messages/<pt|es>/       um JSON por namespace
```

- Componente que conhece um domínio (chama a API, usa tipo do schema, tem texto próprio) fica em `features/<domínio>/components/`; em `shared/ui` só entra o que serviria a qualquer domínio. Nada de `components/`, `utils/` ou `helpers/` soltos em `src/`: se não sabe onde pôr, falta nomear o domínio.
- `page.tsx` não busca dado nem tem lógica: compõe. A busca fica no componente de feature. Feature pode usar componente de outra, sem ciclo; o que duas precisam em comum e é genérico vai para `shared`.
- Arquivo em kebab-case com o nome do que exporta (`member-row.tsx` → `MemberRow`), um componente exportado por arquivo, estilo ao lado em `member-row.module.scss`. Import relativo dentro da feature, `@/features/...` ou `@/shared/...` fora. Sem `index.ts` reexportando (mistura servidor e cliente e atrapalha o bundle).

### Dados e mutações

- Server Component por padrão; `"use client"` só na folha que precisa de estado ou evento (o botão, não a seção).
- Busca: componente async que busca o próprio dado, dentro de `<Suspense>`; busca usada em mais de um lugar vai para `features/<domínio>/server.ts`, com `cache()` do React.
- Mutação de botão: `useApiAction` (`run(chamada, aoDarCerto)` com `pending` e `error` já traduzido). Formulário: `useActionState` + `callApi`. Nunca mostre o código cru da API.
- Server Component não passa função para Client Component (`onClick`, `onSelect`, `renderIcon`, `as`). Lista do servidor com ação por linha: o servidor monta as linhas e põe em cada uma um componente de cliente da feature (`<MemberActions member={m} />`).

### Tempo real

- Página que se atualiza sozinha: `<LiveUpdates>` (`features/realtime`) dizendo o que ela mostra (`tenant`, `tournament`, `match`, `me`), no `params.then(...)` da página ou do `layout.tsx` do grupo. Sem ele, só muda ao navegar. O refresh mantém o `useState` dos componentes de cliente; `key` derivada do dado remonta o bloco.
- O que recarrega cada página: `features/realtime/lib/interest.ts`. Conexão, toasts e som: `features/realtime`; Web Push e service worker: `features/push` e `app/sw.js/route.ts`; sininho: `features/notifications`.

## Estilo (Carbon)

- `globals.scss` importa os estilos do Carbon **por componente** (o pacote inteiro dava 1,25 MB). **Componente do Carbon que não está na lista aparece sem estilo**: acrescente `@use "@carbon/react/scss/components/<pasta>";` (a pasta em `node_modules/@carbon/styles/scss/components`), em ordem alfabética.
- Tema `white` em `:root` e `g100` no escuro (do sistema ou `data-theme="dark"`).
- Toque: abaixo de `md` (672px), os controles `sm` e `md` valem 48px por token em `globals.scss`. Passe o tamanho de desktop (`md`, o padrão; `sm` em linha de tabela); nenhum componente trata.
- O `.scss` passa por um `sass-loader` próprio (`turbopack.rules` em `next.config.ts`): o embutido do Turbopack não acha os `@use` relativos do Carbon. Não volte para o embutido.
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

Cada componente já cumpre os princípios abaixo (foco, `pending`, toque, celular, confirmação): use-o em vez do Carbon cru. Props, exemplos e regras estão no JSDoc de `src/shared/ui/<nome>.tsx`; leia antes de usar. Layout sem regra própria (`Grid`, `Column`, `Stack`, `Layer`, `Tile`, `Form`, `FormGroup`, `SkeletonText`, `InlineLoading`, `AccordionItem`, `SelectItem`, `UnorderedList`/`ListItem`, `CodeSnippet`) vem direto de `@carbon/react`, também em Server Component, enquanto as props forem serializáveis.

Os que não se deduzem do nome:

- **Página**: `Page` (`narrow` para texto e formulário, `wide` para tabela e chave), `PageHeader` (topo de toda página: trilha, título, tags, metadado, ações e ⋯), `PageTabs` (abas que são sub-rotas, no `layout.tsx` do segmento), `TabList` (no lugar do do Carbon, dentro de `Tabs`: sem as setas de rolagem no celular).
- **Ação**: `ButtonLink` (ação que leva a outra página), `ConfirmButton` (ação que não se desfaz fácil), `FormModal` (formulário curto num Modal), `ActionWithNote` (o motivo de não dar, ou o erro, embaixo do botão), `OverflowActions` (o ⋯).
- **Formulário**: `FormSection`, `RequiredLabel`, `TextInput`, `PasswordInput`, `TextArea`, `Select`, `NumberInput`, `DateTimeInput`, `FormError`, `FormActions`. Erro por campo: `FormResult`, `invalidProps`, `formError`, `fieldId` (`shared/lib/form-result.ts`) e `useFocusInvalid`.
- **Listas**: `DataTable` (grande, com busca e paginação), `TileGrid` + `LinkTile` (as listas "minhas"), `StructuredList` (só leitura, em colunas).
- **Estado e conteúdo**: `StatusTag`, `Notice`, `Toast` ("copiado"), `ToastStack` (avisos que chegam sozinhos), `UserText` (texto de usuário sem formatação, com as quebras de linha), `InfoTip` (o (i) ao lado de um termo que nem todos conhecem), `LocalDateTime`/`LocalDate`/`LocalTime`/`LocalDayTime` ("hoje às 21h" no meio de uma frase), `Countdown`, `TimeAgo`.

Formulário: `Form` com `action` do `useActionState`, um `FormSection` por grupo (com um grupo só, os campos direto num `Stack gap={6}`), erro de campo no `invalidText` (`invalidProps`), `FormError` acima dos botões, `FormActions` no fim. Campo travado: `Notice kind="info"` com o motivo **antes** dos campos. Modelo: `features/tenants/components/create-tenant-form.tsx`. Validação: "Formulários: validação" em Princípios de interface; `useFormValidity` (`shared/hooks/`) e os filtros de `shared/lib/input-filters.ts`.

## Textos (messages)

- Um arquivo por namespace de primeiro nível (`messages/pt/Tenant.json` e `messages/es/Tenant.json`); chave nova vai no namespace do seu fluxo, nos dois idiomas. `es` precisa ter as mesmas chaves de `pt` (o `tsc` e o `build` acusam).
- Namespace novo: crie os dois arquivos e acrescente uma linha em cada lista (`pt` e `es`) de `src/i18n/namespaces.ts`, em ordem alfabética.
- `Common` (textos da biblioteca) e `Nav` são da base. Num namespace que outro fluxo também usa, só acrescente chaves: não reordene nem reformate o arquivo.
- Código novo de aviso da API: `Notifications.notices.<código>` (`title` e `body`) nos dois idiomas, os mesmos na página e no service worker. Só `{param}` simples (o service worker não entende plural do ICU); o prazo `confirm_by` entra como `{deadline}`.
- O espanhol é em "tú".

### Termos da interface

| Conceito                                                              | pt      | es      | No código    |
| --------------------------------------------------------------------- | ------- | ------- | ------------ |
| Um jogo só, num mapa                                                  | partida | partida | `game`       |
| Md3, md5 (o confronto entre dois lados)                               | série   | serie   | `match`      |
| O todo                                                                | torneio | torneo  | `tournament` |
| A arena (pool, veto, ban e pick de mapa, mapa decisivo, nome do mapa) | mapa    | mapa    | `map`        |
- "Mapa" só quando é a arena. "Mapa enviado", "rating por mapa", "mapas jogados" e "placar mapa a mapa" são partidas. "Partida pronta", "Partidas que esperam você" e "C1 · partida 3" são séries.
- Concordância: a partida foi enviada, a série foi decidida.
- Outros termos já decididos: "ordem da chave" (seeding, nunca "seed"), "captura de tela", "W.O.", "check-in", "chave", "rating", "todos contra todos" (nunca "round robin"), "queda do K".
- Dois-pontos só em aposto, lista ou rótulo ("Rating: 1500"). Para explicar, ponha ponto e comece outro período: "Faça o check-in na página do torneio. Quem não fizer, não entra em nenhum time." A API também segue isto nos emails e nas mensagens do Discord.

### Como escrever

O texto diz só o que a pessoa não sabe e não vai descobrir logo em seguida. Para cada frase, pergunte o que ela faria diferente sem a frase; se nada, corte. Cortar não é ficar vago: nomeie a coisa ("a série pronta", "o fator K"), dê a data ou o número quando importam e não tire o que a pessoa precisa para decidir (o que se perde, o que não volta, o que falta fazer).

- **Não repita a tela.** Título, etiqueta, botão e coluna já falam: a etiqueta "Inscrição feita" dispensa "Você está inscrito."; o botão "Ver os inscritos" dispensa "Até lá, confira quem está inscrito."; o título "Nova senha" dispensa "Escolha a senha nova."
- **Estado vazio é o estado e a ação**: "Nenhum link de inscrição." e o botão, não "Gere um e mande para quem você quer inscrever…".
- **Fica de fora**: regra interna ("vale para os próximos envios"), mecânica que se percebe ao usar (o pedido de refazer no veto), definição do óbvio ("Completo é o time com 2 jogadores"), o porquê de uma exigência evidente ("A conta precisa de pelo menos um jeito de entrar.") e a exceção de cada caso entre parênteses quando a regra geral basta ("quando abrir vaga", não "(alguém sair ou as vagas aumentarem)").
- **Cada coisa na hora e para quem age**: a consequência de uma ação no modal de confirmação, não acima do botão; o limite no campo e no erro, não na descrição; a validade do convite só para quem gerencia; a explicação longa na página que a usa, não no resumo que leva a ela.
- **Um fato por frase, curta, com ponto no fim.** Sem jargão de sistema ("API", "banco") e sem nada com cara de texto gerado (dois-pontos encadeando explicação, "ou seja", listas de todos os casos).

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
- **Contraste**: só os tokens de cor do tema, que dão AA (4.5:1) em `white` e em `g100`; nunca `opacity` em texto. Confira toda tela nos dois temas.
- **Toque**: abaixo de `md`, controles com 48px (regra global dos tokens; `Link` solto e `LinkTile` também).
- **Foco**: os componentes do Carbon têm o contorno; os controles nativos ganham o do `globals.scss`. Não remova `outline`. Depois de uma ação cujo botão some no refresh, leve o foco para o item afetado (`focusIfLost` de `shared/lib/focus.ts`, com `tabIndex={-1}` no alvo); o Modal e o painel do cabeçalho já devolvem o foco. Campo recusado dentro de acordeão ou aba fechada: abra o que o esconde e use `focusWhenShown`; a validação nativa não abre nada sozinha, por isso o formulário de torneio usa `noValidate` e confere antes de enviar.
- **Ação destrutiva ou que não volta**: `ConfirmButton`, ou item com `confirm` no `OverflowActions` (o Modal diz o que acontece; o primário tem o nome da ação; "Não" recebe o foco). Grave e raro: `confirmSlug`. Nada de `window.confirm`.
- **Celular (390px)**: sem rolagem horizontal fora da chave (o `DataTable` esconde as colunas `secondary`, o `PageHeader` desce as ações, o `TileGrid` vira uma coluna, o `ProgressSteps` fica vertical, títulos longos quebram). O uso principal é no computador, mas parte dos jogadores abre o link pelo celular.
- **Formulários: validação** (o campo cobra o limite; a descrição não: erro depois do envio faz digitar duas vezes, e limite na descrição ninguém lê):
  - Obrigatório: `required` e a estrela vermelha no rótulo (`TextInput`, `PasswordInput`, `TextArea`, `Select`, `NumberInput` e `DateTimeInput` a põem sozinhos; `RequiredLabel` para campo montado à mão), com "obrigatório" para o leitor de tela. Opcional não leva marca.
  - Envio: `useFormValidity()` (`ref` no `<Form>`) dá `canSubmit`, e o botão fica `disabled={!canSubmit}` enquanto faltar obrigatório ou houver campo inválido (desabilitado, e não habilitado com foco no primeiro inválido: o pedido é prevenir o envio, e a causa está à vista). No formulário de torneio (`noValidate`, abre acordeão), a confirmação antes de enviar continua valendo.
  - Caractere que o campo não aceita não entra: `filter` do `TextInput` (`slugChars`, `digitsOnly` em `shared/lib/input-filters.ts`); `maxLength` corta o excesso; `NumberInput` recusa letra, `e`, `+`, vírgula e ponto (e `-` sem `negative`).
  - Valor fora do limite fica inválido com a mensagem curta no `invalidText`: número, na hora (`min`/`max`, "Use de 1 a 10."); texto (`minLength`, `pattern` com `patternMessage`, `type="email"`, `required`, `sameAs` da senha repetida), depois da primeira saída do campo e dali a cada tecla ("mínimo de 3" não fica vermelho na primeira letra).
  - Regra que só o chamador sabe (data à frente de agora): `problem` do `TextInput`/`DateTimeInput`, com a mensagem traduzida; aparece na hora e bloqueia o envio.
  - `helperText` só existe quando o rótulo não basta: diz o que o campo é ou um efeito que não se vê, nunca a faixa nem o tamanho. O erro da API continua no `invalid`/`invalidText` (`invalidProps`) e vale mais que o do campo.
- **Estado vazio e erro**: o `EmptyState` diz o estado e põe a ação, sem ensinar o uso (ver "Como escrever"); erro sempre traduzido, junto do controle que falhou (`invalidText`, `ErrorText`) ou acima dos botões (`FormError`).

## MCP do Carbon

- `code_audit` nos arquivos que você criou ou mudou, em lote. Ignore quando ele sugerir `<Theme>` em volta da página ou classes `cds--type-*`: o tema vem do `globals.scss`, o tipo do mixin `type-style`.
- A versão instalada manda: na dúvida sobre uma prop, confira o `.d.ts` em `node_modules/@carbon/react/lib/components/<Componente>/`.
