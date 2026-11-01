import { DefinitionTooltip as CarbonDefinitionTooltip } from "@carbon/react";
import type { ComponentProps } from "react";

type Props = ComponentProps<typeof CarbonDefinitionTooltip>;

/**
 * Termo sublinhado com pontilhado no meio de um texto (`DefinitionTooltip` do
 * Carbon), que mostra a definição ao clicar ou focar (com `openOnHover`,
 * também ao passar o mouse). `children` é o termo; `definition`, a
 * explicação, só texto e curta. Rótulo de campo ou título com (i) ao lado:
 * `InfoTip`; nome de um botão: `Tooltip`.
 *
 * O Carbon não tem texto próprio aqui e gera o `id`: o componente existe
 * para o import ficar na `shared/ui`. Serve em Server e em Client Component.
 */
export function DefinitionTooltip(props: Props) {
  return <CarbonDefinitionTooltip {...props} />;
}
