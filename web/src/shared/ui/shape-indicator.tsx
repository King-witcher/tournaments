import type { ComponentProps } from "react";

import { preview__ShapeIndicator as CarbonShapeIndicator } from "./carbon/shape-indicator";

type Props = ComponentProps<typeof CarbonShapeIndicator>;

/**
 * Nível de gravidade com forma colorida e texto (`ShapeIndicator` do Carbon,
 * ainda em prévia): de `failed` e `critical` a `low`, mais `stable`,
 * `informative`, `incomplete` e `draft`. A forma muda com o nível, para não
 * depender só da cor. `label` é o texto já traduzido; `compact` mostra só a
 * forma, com o `label` numa dica e para o leitor de tela; `textSize`, 12 |
 * 14. Estado de um processo (em andamento, pendente): `IconIndicator`; com
 * fundo colorido: `StatusTag`.
 *
 * O Carbon não tem texto próprio aqui: o componente existe para o import
 * ficar na `shared/ui`. Serve em Server e em Client Component.
 */
export function ShapeIndicator(props: Props) {
  return <CarbonShapeIndicator {...props} />;
}
