# Ações e formulários: botões, Modal, Forms e campos

## Botões

**Hierarquia** (do mais forte ao mais fraco):

- **Primary**: a principal chamada para ação da tela. **Uma por tela**, sem contar o cabeçalho, um Modal ou um painel lateral (fluxos temporários têm a sua). Nem toda página precisa de uma: página que serve para ler ou examinar usa tertiary e ghost.
- **Secondary**: a ação secundária, em geral junto da primária ("Cancelar", "Voltar").
- **Tertiary**: sozinha ou com uma primária quando há mais de uma chamada; subtarefas da página; ações do topo da página; a ação de um estado vazio.
- **Ghost**: a mais discreta; ações de tabela e de barras de ferramenta, "Cancelar" num fluxo em passos (primária para avançar, secundária para voltar, ghost para cancelar).
- **Danger**: ação destrutiva (excluir, remover), em versão primária, tertiary ou ghost. Nunca só com ícone.

**Grupos**: dois ou três botões. Mais de três ações sobre o mesmo dado vão para um Menu button ou uma toolbar.

**Alinhamento**: em página inteira (formulário de página), o grupo fica à esquerda, com o primário à esquerda do secundário. Em Modal, assistente e painel lateral, à direita, com o primário à direita. Botão fluido (que ocupa uma fração do container) nunca fica à esquerda: alinha à direita ou ocupa a largura toda.

**Rótulo**: verbo + substantivo ("Criar torneio", "Enviar resultado"), sentence case; exceções para ações comuns e curtas ("Salvar", "Cancelar", "Fechar", "Adicionar", "Excluir"). Nunca só um substantivo.

**Ícone**: só quando ajuda (ícone de abrir fora para link externo, "+" em adicionar), o mesmo estilo em todo o grupo. Botão só com ícone (em geral ghost) leva o rótulo numa dica; danger não pode ser só ícone.

**Tamanhos**: `sm` 32, `md` 40, `lg` 48, `xl` 64, `2xl` 80 px; carregando, o botão mostra o inline loading no lugar do rótulo.

No projeto: `Button` (`md` por padrão, `pending` sem `disabled`), `ButtonLink`, `ConfirmButton`, `ActionWithNote`, `FormActions` (primário à esquerda e "Cancelar" ao lado), `IconButton` sempre com `label` (`web/CLAUDE.md`, "Ação" e "Pending").

## Modal

**Quando**: exigir resposta imediata, avisar algo urgente, confirmar uma decisão. **Quando não**: a pessoa precisa consultar a página por trás (é um diálogo não modal ou um painel); nunca Modal dentro de Modal; nunca Modal do tamanho da página. Diálogo só por iniciativa da pessoa e com tarefa curta e focada.

**Variantes**:

- **Passive**: só informa, sem ação; o X recebe o foco.
- **Transactional**: precisa de uma ação para fechar; "Cancelar" e a primária. Numa confirmação ou decisão, o foco começa na primária.
- **Danger**: transactional para o que é destrutivo ou não volta; o foco começa em "Cancelar", não no botão vermelho.
- **Acknowledgment**: um botão só ("OK").
- **Progress**: passos; cancelar, anterior e próximo.

**Tamanhos**: `xs`, `sm`, `md`, `lg`, com altura máxima de 48%, 72%, 84% e 96% da tela; o conteúdo rola por dentro.

**Conteúdo**: o título diz a tarefa ou é a própria pergunta; o corpo diz o que vai acontecer; a primária repete o verbo do que abriu o Modal. Evite dentro dele o que leva para fora da tarefa (links de navegação), o que esconde informação (acordeão) e tabela com interação complexa. Validação no próprio Modal; ao concluir, fecha e devolve o foco ao gatilho.

No projeto: nunca `Modal` do Carbon direto numa feature: `ConfirmButton`, `ConfirmModal`, `FormModal`, `PassiveModal`. Em toda confirmação o "Não" recebe o foco, e o primário tem o nome da ação (`docs/decisions/interface.md`, "Confirmação em Modal" e "Conferência contra o Carbon (31/10)").

## Exclusão (padrão Common actions)

- **Baixo impacto** (desfazer ou refazer é trivial): exclui no clique, sem aviso.
- **Médio impacto** (não volta, ou o dado não se refaz fácil; ou várias coisas de uma vez): pede confirmação dizendo o que acontece.
- **Alto impacto** (refazer custaria muito, apaga muito, ou leva outras coisas junto): Modal e a pessoa digita o nome do recurso.
- **Depois**: volte à lista de onde o item saiu, tire o item da lista e mostre o sucesso; se falhar, avise.
- **Remover x excluir**: remover tira de um grupo ou container (o item continua existindo); excluir destrói.

No projeto: `ConfirmButton` ou item com `confirm` no `OverflowActions`; o grave e raro com `confirmSlug` (desfazer o início, transferir a posse, excluir) (`web/CLAUDE.md`, "Ação destrutiva").

## Formulários (padrão Forms)

**Layout**: uma coluna sempre que der. Duas ou três colunas só para campos com relação lógica, em colunas proporcionais à grade; uma linha de duas colunas se comporta como grupo (se um campo ganha a mensagem de erro, o vizinho cresce junto). Entre colunas, o gutter wide (32 px); entre campos empilhados, 32 px. Agrupe por assunto, com título de grupo. Formulário longo: seções e divulgação progressiva, com cuidado ao esconder em acordeão.

**Rótulo**: sempre visível, curto, sentence case, sem dois-pontos no fim. Placeholder não substitui o rótulo (some ao digitar e tem pouco contraste). Valor padrão quando há um provável.

**Helper text**: abaixo do campo, quando o rótulo não basta (formato, o que o campo afeta). Dica longa vai numa toggletip ao lado do rótulo.

**Obrigatório x opcional**: marque a minoria. Se quase tudo é obrigatório, marque só os opcionais com "(opcional)"; se quase tudo é opcional, marque os obrigatórios com "(obrigatório)". Use o mesmo critério em todo o produto.

**Validação**: o estado inválido tem borda vermelha, ícone e mensagem embaixo do campo; a mensagem diz o que está errado e como corrigir. Validação em tempo real quando ajuda a manter o dado limpo; senão, ao enviar. Aviso (warning) para o que é aceito mas merece atenção. O padrão Forms tem ainda uma parte sobre habilitar e desabilitar o botão de enviar; consulte-a no MCP se a regra do projeto (abaixo) for revista.

**Só leitura x desabilitado** (padrões Read-only e Disabled states):

- **Só leitura**: o valor importa e precisa ser lido, mas não pode mudar agora. Mantém contraste e é lido pelo leitor de tela.
- **Desabilitado**: o controle está indisponível até a pessoa fazer algo (completar outro campo, escolher uma opção). Não passa contraste e o leitor de tela pula; não use para o que deve ser lido.
- Não troque um pelo outro, nem passe um campo de desabilitado para só leitura.

No projeto: obrigatório com a estrela vermelha e "obrigatório" para o leitor de tela, opcional sem marca; o envio fica desabilitado enquanto faltar obrigatório ou houver campo inválido; caractere recusado não entra (`filter`); número fora da faixa fica vermelho na hora, texto depois da primeira saída do campo; `helperText` só quando o rótulo não basta, nunca a faixa (`web/CLAUDE.md`, "Formulários: validação"). Campo travado: `readOnly` e o motivo num `Callout` antes dos campos. Peças: `Form`, `FormSection`, `PageFormLayout`, `FormRow`, `FormError` acima de `FormActions`, `invalidProps`, `useFormValidity`, `LeaveGuard`. Formulários enviam por `onSubmit` (`docs/decisions/interface.md`).

## Campos

**Text input e Text area**: uma linha ou várias. Tamanhos `sm` 32, `md` 40, `lg` 48 px. Contador de caracteres quando há limite. Password input com o botão de mostrar.

**Select, Dropdown, Combo box, Multiselect**:

| Componente      | Quando                                                                                     |
| --------------- | ------------------------------------------------------------------------------------------ |
| **Select**      | Escolher um valor que será enviado num formulário; lista nativa do navegador, mais fácil no celular |
| **Dropdown**    | Escolher para agir, filtrar ou ordenar o conteúdo; lista desenhada pelo Carbon             |
| **Combo box**   | Lista longa: digita para filtrar, ou aceita um valor próprio                               |
| **Multiselect** | Vários valores; com filtro se a lista é longa                                              |

Duas opções não pedem dropdown: radio. Experiência baseada em formulário ou muito usada no celular: prefira o Select. Opções em ordem que faça sentido (alfabética, frequência, natural).

**Number input**: ajustes pequenos com os botões de mais e menos. **Não use** para valores que mudam muito (de 1 a 30 seria clique demais: Slider) nem para valores contínuos e exatos numa faixa larga (preço, distância: Text input).

**Date picker**: entrada simples (só campos) para data que a pessoa sabe de cor (nascimento); calendário de uma data ou intervalo quando importa a relação com outros dias ou para agendar. Time picker para hora, com o formato do local e o seletor de fuso quando a hora é marcada.

**Radio, Checkbox, Toggle, Selectable tile**:

- **Radio**: opções exclusivas, exatamente uma.
- **Checkbox**: zero, uma ou várias; também para filtro, ação em lote, aceite de termos e listas com subitens (pai com estado indeterminado).
- **Toggle**: ligar ou desligar com efeito imediato, sem botão de enviar. Padrão com rótulo e o texto do estado; pequeno, em linha com outros componentes, sem rótulo.
- **Selectable tile**: a escolha precisa de mais que um rótulo (descrição, imagem).

No projeto: `TextInput` (com `filter` e `problem`), `PasswordInput`, `TextArea`, `MarkdownField`, `Select` (nativo, poucas opções), `Dropdown`, `ComboBox`, `MultiSelect`, `NumberInput`, `DatePicker`, `DateRangePicker`, `TimePicker`, `DateTimeInput` (24 horas, sem AM/PM), `RadioGroup` (empilhado sempre que der), `CheckboxGroup`, `Toggle`, `RadioTileGroup`, `SelectableTile`, `Slider`, `Search`, `FileUploader`, `FileDropZone` (`web/CLAUDE.md`, "Formulário"; `docs/decisions/interface.md`, "Data e hora como no Carbon").
