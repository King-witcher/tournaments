import { Tooltip as CarbonTooltip } from "@carbon/react";
import type { ComponentProps } from "react";

type Props = ComponentProps<typeof CarbonTooltip>;

/**
 * Dica curta de um controle (`Tooltip` do Carbon), ao passar o mouse ou
 * focar. `label` é o nome do controle (o leitor de tela lê só ele: para um
 * botão sem texto); `description` complementa o texto que o controle já tem
 * (lida junto dele). O filho é um só elemento focável. A dica não leva link
 * nem ação e é só texto curto. Conteúdo que precisa abrir no clique ou no
 * toque, ou com link: `InfoTip` (Toggletip); termo no meio de uma frase:
 * `DefinitionTooltip`; botão só com ícone: `IconButton`, que já tem a sua.
 * `align` escolhe o lado; `autoAlign` troca se não couber; `enterDelayMs` e
 * `leaveDelayMs` mudam a espera para abrir e fechar.
 *
 * O Carbon não tem texto próprio aqui: o componente existe para o import
 * ficar na `shared/ui`. Serve em Server e em Client Component.
 */
export function Tooltip(props: Props) {
  return <CarbonTooltip {...props} />;
}
