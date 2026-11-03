import { Children, isValidElement, type ReactNode } from "react";

import { cn } from "@/shared/lib/cn";
import styles from "./tile-grid.module.scss";

/**
 * Grade das listas "minhas" (seus torneios, suas comunidades): cada filho,
 * em geral um `LinkTile`, vira um item. As colunas saem da largura que a grade
 * tem, então ela serve na página inteira e numa coluna estreita (`PageColumns`):
 * uma no celular, até quatro numa tela larga. `stacked`: sempre uma, com os
 * tiles quase colados, para os `ActionTile`.
 */
export function TileGrid({
  children,
  label,
  stacked,
}: {
  children: ReactNode;
  /** Nome da lista para leitor de tela, quando não há título logo acima. */
  label?: string;
  /** Uma coluna só, para uma lista de tarefas. */
  stacked?: boolean;
}) {
  return (
    <ul
      className={cn(styles.grid, stacked && styles.stacked)}
      aria-label={label}
    >
      {Children.toArray(children)
        .filter(isValidElement)
        .map((child) => (
          <li key={child.key} className={styles.item}>
            {child}
          </li>
        ))}
    </ul>
  );
}
