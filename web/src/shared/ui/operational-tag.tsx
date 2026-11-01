"use client";

import { OperationalTag as CarbonOperationalTag } from "@carbon/react";
import type { ComponentProps } from "react";

type Props = ComponentProps<typeof CarbonOperationalTag>;

/**
 * Etiqueta que é um botão (`OperationalTag` do Carbon): abre os detalhes do
 * que ela resume, em geral um `Popover` com a lista ("+3 mapas"). `onClick`
 * faz a ação; `text` é o texto; `type`, a cor; `size`, `sm` | `md` | `lg`;
 * `renderIcon`, o ícone antes do texto. Etiqueta só de leitura: `StatusTag`;
 * que marca uma escolha: `SelectableTag`; que se tira: `DismissibleTag`.
 *
 * O Carbon não tem texto próprio aqui e gera o `id`: o componente existe
 * para o import ficar na `shared/ui`.
 */
export function OperationalTag(props: Props) {
  return <CarbonOperationalTag {...props} />;
}
