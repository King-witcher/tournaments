# Acessibilidade e conteúdo

## Teclado e foco

- **Tudo pelo teclado.** Todo elemento interativo é alcançável por Tab e ativado por Enter ou Espaço; os componentes do Carbon já trazem a interação. Elemento nativo (`button`, `a`, `input`) antes de ARIA; ARIA só para completar o que o HTML não diz.
- **Ordem do foco** lógica e previsível, dada pela ordem do código: cabeçalho, navegação principal, navegação do conteúdo, conteúdo (da esquerda para a direita, de cima para baixo), rodapé. Não reordene com CSS o que muda o sentido.
- **Foco visível** sempre: o Carbon desenha o contorno com `$focus` (2 px). Nunca remova `outline`.
- **Regiões e títulos**: cabeçalho, navegação, conteúdo principal e rodapé como landmarks; títulos em ordem (h1, h2, h3) para quem navega por eles. `SkipToContent` como primeiro item.
- **Por componente** (o que o Carbon já faz e não se deve quebrar):
  - Modal prende o foco dentro dele e devolve ao gatilho ao fechar; Esc fecha. Passivo começa no X; confirmação, na primária; destrutivo, em "Cancelar".
  - Menus (overflow, menu button, combo button): Enter ou Espaço abre com o foco no primeiro item; setas andam; Esc fecha e volta ao botão.
  - Abas: Tab entra na aba ativa, setas trocam.
  - Combo box: digitar filtra, setas abrem e andam, Enter escolhe, Esc limpa e fecha (Espaço digita, não escolhe).
  - Tabela: cabeçalho ordenável por Tab e Enter ou Espaço; paginação é um componente à parte.
  - Notificação inline e toast não recebem foco; a actionable prende o foco até a resposta.
  - Menu lateral atrás do botão no celular (ou com zoom perto de 175%): o botão abre por cima do conteúdo e vira X.

No projeto: depois de uma ação cujo botão some no refresh, o foco vai para o item afetado (`focusIfLost`); campo recusado dentro de acordeão ou aba fechada abre o que o esconde (`focusWhenShown`); pendente é `aria-disabled`, não `disabled`, para não perder o foco (`web/CLAUDE.md`, "Foco"; `docs/decisions/interface.md`, "Foco depois de agir").

## Alvos de toque

Alvo interativo com pelo menos 44 px; o Carbon centra o ícone de 20 a 22 px num alvo de 48 px. Ícone clicável ganha padding até o tamanho.

No projeto: abaixo do `md`, os controles `sm` e `md` valem 48 px por token em `globals.scss`; `Link` solto e `LinkTile` também (`web/CLAUDE.md`, "Toque").

## Rótulos e alternativas

- Todo campo tem rótulo visível ligado a ele; placeholder não é rótulo.
- Botão só com ícone tem nome acessível (o `label`, que vira a dica).
- O texto do link diz o destino; links genéricos repetidos ("saiba mais") precisam de contexto programático.
- Imagem que informa tem texto alternativo; a decorativa, alt vazio.
- Estado nunca só pela cor: ícone, forma ou texto junto (padrão Status indicators).
- O que precisa ser lido não fica desabilitado (desabilitado não passa contraste e o leitor de tela pula): use só leitura.
- Mudança que chega sem a pessoa pedir (toast, contagem) é anunciada por uma região viva sem roubar o foco.

No projeto: os componentes da `shared/ui` já traduzem os textos que o Carbon põe em inglês (aria-label "Close", "Clear search input"); usando um do Carbon direto, passe a prop traduzida (`web/CLAUDE.md`, "O que não fazer").

## Escrita (diretrizes de conteúdo da IBM)

Base do Carbon: Language and grammar, Capitalization, Terminology, Punctuation e Numbers, do IBM Style.

- **Sentence case** em tudo: títulos, rótulos, botões, abas, cabeçalhos de coluna, avisos. Só a primeira palavra e os nomes próprios com maiúscula.
- **Claro, curto e acionável**: voz ativa, presente, segunda pessoa; uma ideia por frase; sem jargão de sistema.
- **Ações**: verbo + substantivo ("Criar comunidade", "Copiar link de convite"); ações comuns ficam no verbo ("Salvar", "Cancelar", "Fechar", "Excluir"). O mesmo verbo do botão que abre no título e no primário do Modal.
- **Termos padronizados**: um conceito, um nome, em toda a interface.
- **Erro**: diga o que aconteceu e como resolver, sem culpar a pessoa e sem código técnico; junto do que falhou (no campo, no bloco) e, se for do envio, perto da ação.
- **Estado vazio**: o estado e o caminho para sair dele (a ação).
- **Confirmação**: o título e o corpo dizem o que vai acontecer e o que não volta; o botão nomeia a ação, não "OK" nem "Sim".
- **Números**: algarismos na interface (contagens, limites, posições), com o separador do idioma; intervalos com travessão curto ("1–25"); unidade junto do número.
- **Datas e horas**: no formato do idioma da pessoa, sem ambiguidade; relativas ("há 5 minutos") quando a proximidade importa mais que a data.
- **Pontuação**: rótulo sem dois-pontos no fim; título sem ponto final; frase de corpo com ponto.

No projeto: as regras de `web/messages/CLAUDE.md` ("Termos da interface" e "Como escrever") valem mais que estas quando divergem: o texto diz só o que a pessoa não sabe e não vai descobrir logo em seguida; o estado vazio não ensina o uso; dois-pontos só em aposto, lista ou rótulo ("Rating: 1500"); termos fixos (partida, série, torneio, mapa). Datas e horas pelos componentes `LocalDateTime`, `LocalDate`, `LocalTime`, `LocalDayTime`, `TimeAgo` e `Countdown`; os formatos do inglês (en-US) estão em `docs/decisions/stack.md`, na linha dos idiomas. Textos sempre em `messages/<idioma>/`, nos três idiomas.
