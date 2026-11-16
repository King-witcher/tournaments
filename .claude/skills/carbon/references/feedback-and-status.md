# Retorno e estado: avisos, indicadores, vazio, carregamento, progresso

## Notificações

Quatro variantes, escolhidas pelo momento e por quem dispara:

| Variante       | Quando                                                                                       | Fecha                         | Foco |
| -------------- | -------------------------------------------------------------------------------------------- | ----------------------------- | ---- |
| **Inline**     | Retorno de uma tarefa, perto do que a causou (o formulário, a seção); erro, sucesso, aviso   | Pela pessoa, ou quando o problema se resolve | Não recebe |
| **Toast**      | Evento do sistema que não depende da tela atual; mensagem curta, no topo, some em segundos   | Sozinho (ou pelo X)           | Não recebe |
| **Actionable** | Aviso com o estilo de inline ou toast que pede uma resposta (botão ou link)                  | Pela ação ou pelo X; Esc      | Prende o foco até a resposta |
| **Callout**    | Informação importante que chega junto com a página, no lugar onde vale                       | Não fecha                     | Não recebe |

- **Tipos**: `error`, `warning` (e `warning-alt`), `success`, `info` (e `info-square`). Cada um com borda e ícone em `$support-*`.
- **Contraste**: baixo (fundo claro `$notification-*-background-color`) ou alto (fundo invertido). Escolha um por produto; baixo contraste combina com aviso dentro da página.
- **Conteúdo**: título curto que diz o fato; corpo opcional com o que fazer; ação com verbo. Toast pode levar a hora do evento.
- **Notificação x Modal**: o Modal é para o que exige resposta imediata e interrompe; a notificação informa sem bloquear.
- Inline e toast não têm elemento interativo e não recebem foco; com botão ou link, é a actionable.

No projeto: `Notice` aplica a regra: `Callout` para informação e cuidado que carregam com a página (motivo de um campo travado, antes dele; exclusão agendada), `InlineNotification` para erro e sucesso, o retorno de algo que aconteceu (`docs/decisions/interface.md`, "Conferência contra o Carbon (31/10)"). `TaskNotice` e `TaskNoticeList` para a pendência de quem vê, com o botão que a resolve; `ActionableNotification`; `Toast` ("copiado", 5 s) e `ToastStack` (avisos que chegam sozinhos, no canto de cima à direita, 20 s mesmo com link). Na home, a actionable usa `role="status"` em vez de prender o foco (`docs/decisions/interface.md`, "Aviso de login de reserva").

## Status indicators (padrão)

Mostram o estado de um item com ícone, forma, cor e texto. Variantes:

- **Icon indicator**: ícone (forma, cor e símbolo) e rótulo em texto. Quando há espaço e o estado pede atenção: notificações, passos, tabelas, listas de tarefas, widgets de dashboard.
- **Shape indicator**: só forma e cor, com rótulo. Espaço pequeno ou muito dado para varrer com o olho: listas, tabelas, gráficos, diagramas de rede.
- **Badge indicator**: contagem (avisos não lidos) ou um ponto de "há novidade".
- **Differential indicator**: subida ou descida de um número (seta e cor).

**Níveis de atenção**, definidos pelo produto:

- **Alta**: ação imediata por irregularidade, falha ou algo destrutivo (erros, alertas, exceções, confirmações).
- **Média**: sem ação imediata; retorno de uma ação (confirmações de recebimento, progresso).
- **Baixa**: algo pronto para ver, retorno do sistema, mudou desde a última visita.

**Regras**:

- Vários estados consolidados num só: vale a cor de maior atenção.
- Sem ação a tomar ou sem importância para destacar: texto simples, sem indicador. Mais de cinco ou seis indicadores na tela sobrecarregam.
- Cor nunca sozinha: a forma e o texto também dizem. Ícone sozinho é aceitável com o rótulo para leitor de tela e numa dica.
- Formas têm associação aprendida (hexágono para parar, triângulo para cuidado); não troque o significado.

No projeto: `IconIndicator` e `ShapeIndicator`; o processo que pede cuidado sem ser erro é `caution-minor`; o esperado é texto e a `StatusTag` fica para a exceção (`docs/decisions/interface.md`, "Etiqueta só na exceção"). O número do sininho é o `BadgeIndicator`.

## Estado vazio (padrão)

Aparece quando não há dado: primeiro uso, conteúdo apagado ou indisponível.

- **Tipos**: sem dado (primeiro uso: o que apareceria ali e como começar); resultado de ação do usuário (busca ou filtro sem resultado, lista esvaziada); erro (não carregou, sem permissão), com o caminho para resolver.
- **Anatomia**: título (o estado), corpo opcional (por que está vazio e o próximo passo), ação (botão ou link) e, em espaços grandes, uma ilustração; em espaços pequenos, só texto.
- Alternativas para o primeiro uso: documentação no lugar, onboarding, conteúdo inicial pronto.

No projeto: `EmptyState` é o estado e a ação, sem ensinar o uso e sem ilustração (`web/CLAUDE.md`, "Estado vazio e erro"; `web/messages/CLAUDE.md`, "Como escrever"). Busca sem resultado troca a tabela inteira pelo estado vazio.

## Carregamento (padrão Loading)

| Situação                                                                 | Use                     |
| ------------------------------------------------------------------------ | ----------------------- |
| Primeira carga de uma página ou bloco com forma conhecida                | Skeleton                |
| Um componente processa uma ação (convite enviado, item salvo)            | Inline loading          |
| A aplicação toda fica indisponível enquanto processa, ou passa de alguns segundos | Loading (tela cheia, com overlay) |
| O progresso pode ser medido (envio de arquivo, processamento longo)      | Progress bar            |
| Dashboard ou tela com várias fontes                                      | Progressive loading     |

- **Skeleton**: versão simplificada do componente, com movimento para mostrar que não travou; dura poucos segundos. Só em componentes de container ou de dado (tiles, structured lists, tabelas, cards). Botões e campos em geral não precisam. Nunca em toast, overflow menu, item de dropdown, Modal ou indicador de carregamento; o conteúdo de um Modal pode ter, o Modal não.
- **Progressive loading**: primeiro a estrutura (esqueletos dos containers e do texto que é dado, mais o texto fixo); depois imagens, o que está fora da tela e as ações. O que não tem esqueleto pode ser espaço vazio do tamanho final (uma imagem 600 x 600 é um vazio 600 x 600 até chegar).
- **Inline loading**: estados inativo, ativo, concluído e erro, com rótulo opcional ("Enviando…", "Enviado").
- Indicador de carregamento só diz que algo acontece; se vai demorar mais que um instante, mostre o progresso.

No projeto: toda rota que busca dado tem `loading.tsx` com esqueletos na forma dela, e a parte lenta fica num `<Suspense>` com esqueleto do mesmo tamanho (`web/CLAUDE.md`, "Carregamento"). Esqueletos prontos em `shared/ui/skeleton.tsx` (`PageSkeleton`, `PageHeaderSkeleton`, `SectionSkeleton`, `TableSkeleton`, `TileGridSkeleton`, `FormSkeleton`, `TabLoading`...); botão que chama a API recebe `pending` (o indicador dentro do botão); `Loading` para a espera sem forma conhecida.

## Progress bar

- **Determinada** (a porcentagem é conhecida) ou **indeterminada** (sabe-se que avança, não quanto). Exemplos: download, upload, processamento de dado.
- **Quando não**: a espera vai mostrar conteúdo (skeleton); o avanço depende de passos da pessoa (Progress indicator); o progresso é medido por ações do usuário, como espaço usado (é um Meter); dura menos de 5 s (Loading).
- Estados: ativo, sucesso, erro. No erro, a barra ocupa a largura toda em vermelho, fica visível e o erro é explicado numa inline notification ou no formulário. Terminado, a barra fica como confirmação ou some, conforme o caso.
- Rótulo (o que carrega) e helper text (quanto, quanto falta).

No projeto: `ProgressBar` (o aviso falado traduzido).

## Progress indicator

Guia uma tarefa linear em vários passos, mostrando os feitos, o atual e os seguintes. Horizontal ou vertical. Não é aba: as abas não têm ordem nem estado de concluído. Passos com rótulo curto e, se preciso, helper text; um passo com erro mostra o ícone de erro.

No projeto: `ProgressSteps`: só leitura no ciclo do torneio, navegável na série e na sala de veto; vertical sem espaço e, com mais de quatro passos no celular, um `Dropdown` (`docs/decisions/interface.md`, "Passos").
