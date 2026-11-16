# Cor, temas e camadas

## Temas

Quatro temas, dois claros e dois escuros. O nome do token não muda entre eles; o valor muda.

| Tema   | `$background` | `$layer-01` | `$layer-02` | `$layer-03` |
| ------ | ------------- | ----------- | ----------- | ----------- |
| `white`| branco        | gray 10     | branco      | gray 10     |
| `g10`  | gray 10       | branco      | gray 10     | branco      |
| `g90`  | gray 90       | gray 80     | gray 70     | gray 60     |
| `g100` | gray 100      | gray 90     | gray 80     | gray 70     |

Nos temas claros as camadas alternam a cada nível; nos escuros, cada nível é um passo mais claro.

- **Preferência do sistema**: sem escolha da pessoa, siga o claro ou escuro do sistema; o par que o Carbon usa no exemplo é `g10` e `g100`. Ofereça um controle de modo quando fizer sentido e respeite a escolha.
- **Tema em linha**: uma região pode ter outro tema (o `Theme` do React, ou o mixin `theme` numa zona do CSS). Serve para o shell ou uma faixa de destaque; não para alternar tema dentro do mesmo bloco.

No projeto: `g10` em `:root`, `g100` no escuro do sistema ou com `data-theme="dark"`, casca `white` no claro e `g90` no escuro por zona (`shared/ui/_shell-zone.scss`); escolha no painel da pessoa, em `localStorage` (`docs/decisions/interface.md`, "Tema"). Ignore a sugestão do `code_audit` de pôr `<Theme>` em volta da página.

## Camadas

O modelo de camadas empilha superfícies: página (`$background`), container sobre ela (`$layer-01`), container dentro dele (`$layer-02`) e assim por diante. Cada nível contrasta com o de baixo.

- **Tokens de camada explícitos** (`$layer-01..03`, `$field-01..03`, `$border-subtle-01..03`, `$layer-accent-01..03`): valor fixo por tema; servem quando o componente sempre fica no mesmo nível.
- **Tokens contextuais** (`$layer`, `$field`, `$border-subtle`, `$border-strong`, `$layer-accent`, `$layer-hover`, `$layer-selected`...): o valor depende de quantos `Layer` envolvem o elemento. Os componentes do Carbon usam estes; por isso um `TextInput` dentro de um `Layer` pinta o campo com `$field-02` sem prop nenhuma.
- **`Layer`** (React) sobe um nível para tudo que está dentro. Use-o em volta do conteúdo de um container que tem fundo de camada, para os campos, tiles e tags de dentro contrastarem com ele. Até três níveis.
- **`$field`**: fundo de campo de formulário. O campo fica sempre um nível acima do que está atrás dele.
- **`$layer-accent`**: um tom de destaque sobre a camada atual, para partes de componente que precisam se separar do fundo (aba contida não escolhida, por exemplo).
- **`$border-subtle`**: divisória entre itens da mesma camada (linhas de tabela, separador de lista, borda de tile em grade condensada). `$border-strong`: a linha de baixo do campo e bordas que identificam um controle. `$border-tile`: borda do tile clicável e selecionável.

No projeto: `Panel` é o container branco e já traz o `Layer`; componente que pode ficar dentro de um `Panel` usa o token contextual (`$layer`, não `$layer-01`) (`web/CLAUDE.md`, "Estilo"). O que tem fundo próprio (tabela, chave, lista de tiles) fica solto sobre o cinza.

## Tokens que mais aparecem

Nome = elemento + papel (+ estado): `$text-secondary`, `$background-hover`.

**Texto**: `$text-primary` (corpo, títulos), `$text-secondary` (rótulos, metadado, dica, cabeçalho de coluna), `$text-placeholder`, `$text-helper`, `$text-error` (mensagem de erro do campo), `$text-on-color` (sobre fundo colorido: botão primário, barra de ação em lote), `$text-inverse` (sobre fundo invertido: tooltip, toast de alto contraste), `$text-disabled`.

**Link**: `$link-primary`, `$link-primary-hover`, `$link-visited` (opcional), `$link-inverse`.

**Ícone**: `$icon-primary`, `$icon-secondary` (ícone de apoio, decorativo de componente), `$icon-interactive` (ícone que é o próprio alvo, seta do tile clicável), `$icon-on-color`, `$icon-inverse`, `$icon-disabled`. O ícone segue a cor do texto que acompanha.

**Suporte** (estado; nunca decoração): `$support-error`, `$support-success`, `$support-warning`, `$support-info`, as versões `-inverse` para fundo invertido, e `$support-caution-minor`, `$support-caution-major`, `$support-caution-undefined` para os níveis do padrão Status indicators. Os fundos de notificação de baixo contraste são `$notification-*-background-color`.

**Foco e interação**: `$focus` (contorno de 2 px), `$focus-inset`, `$focus-inverse`; `$background-hover`, `$background-active`, `$background-selected`; `$layer-hover`, `$layer-active`, `$layer-selected`, `$layer-selected-inverse` (marca forte do escolhido); `$interactive` (marca de interação genérica); `$overlay` (fundo atrás de Modal); `$highlight`.

**Marca e destaque**: `$background-brand` (barra de ação em lote da tabela), `$background-inverse` (alto contraste: tooltip, tag selecionada, content switcher de alto contraste).

**Tags**: cada cor de tag tem tokens próprios de componente; nos claros, texto e ícone no passo 70 e fundo no 20; nos escuros, texto no 20 e fundo no 70. Alto contraste e outline usam tokens de núcleo.

No projeto: no Sass, `theme.$text-secondary` (`@use "@carbon/react/scss/theme"`); nunca cor da paleta (`colors.$blue-60`) nem hex.

## Contraste

- Texto menor que 24 px: 4.5:1. Texto de 24 px para cima: 3:1. Elementos gráficos (ícone que informa, borda que identifica um controle, marca de gráfico): 3:1.
- Ícone passa o mesmo contraste do texto (4.5:1) e usa a mesma cor dele.
- Os pares de tokens do tema (texto sobre a camada certa) já cumprem esses números; o contraste quebra quando se mistura um token de uma camada com o fundo de outra, ou quando se inventa cor.
- Estado desabilitado não precisa cumprir contraste, e por isso não serve para o que tem de ser lido (veja `forms-and-actions.md`, "Só leitura x desabilitado").
- Na paleta da IBM, a distância em passos garante o contraste: com o passo 50 ou 60, quatro a seis passos de distância dão 4.5:1.

No projeto: só tokens do tema, nunca `opacity` em texto, e toda tela conferida em `g10` e `g100` (`web/CLAUDE.md`, "Contraste").
