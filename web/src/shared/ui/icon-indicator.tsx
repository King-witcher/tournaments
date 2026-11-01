import { preview__IconIndicator as CarbonIconIndicator } from "@carbon/react";
import type { ComponentProps } from "react";

type Props = ComponentProps<typeof CarbonIconIndicator>;

/**
 * Estado com ícone colorido e texto (`IconIndicator` do Carbon, ainda em
 * prévia), para quando a forma do ícone, além da cor, distingue o estado:
 * falhou, em andamento, pendente, não começou. Cabe numa célula de tabela ou
 * numa lista. `kind` é o estado; `label`, o texto já traduzido; `compact`
 * mostra só o ícone, com o `label` numa dica e para o leitor de tela; `size`,
 * 16 | 20. Estado com fundo colorido, que chama mais atenção: `StatusTag`;
 * nível de gravidade (de crítico a baixo): `ShapeIndicator`.
 *
 * O Carbon não tem texto próprio aqui: o componente existe para o import
 * ficar na `shared/ui`. Serve em Server e em Client Component.
 */
export function IconIndicator(props: Props) {
  return <CarbonIconIndicator {...props} />;
}
