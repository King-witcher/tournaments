"use client";

import { SelectableTag as CarbonSelectableTag } from "@carbon/react";
import type { ComponentProps } from "react";

type Props = ComponentProps<typeof CarbonSelectableTag>;

/**
 * Etiqueta que liga e desliga (`SelectableTag` do Carbon, com
 * `aria-pressed`): filtros de escolha múltipla, lado a lado dentro de um
 * `FormGroup`, que dá o nome comum. `selected` + `onChange(selected)`
 * controlam de fora; `defaultSelected` deixa o estado com ela. `text` é o
 * texto; `size`, `sm` | `md` | `lg`; `renderIcon`, o ícone antes do texto.
 * Escolha dentro de um formulário: `Checkbox` ou `RadioGroup`; etiqueta que
 * se tira: `DismissibleTag`.
 *
 * O Carbon não tem texto próprio aqui e gera o `id`: o componente existe
 * para o import ficar na `shared/ui`.
 */
export function SelectableTag(props: Props) {
  return <CarbonSelectableTag {...props} />;
}
