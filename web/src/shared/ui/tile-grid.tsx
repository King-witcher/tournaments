import { Children, isValidElement, type ReactNode } from "react";

import styles from "./tile-grid.module.scss";

/**
 * Grade das listas "minhas" (seus torneios, suas comunidades): cada filho,
 * em geral um `LinkTile`, vira um item. As colunas saem da largura que a grade
 * tem, então ela serve na página inteira e numa coluna estreita (`PageColumns`):
 * uma no celular, até quatro numa tela larga.
 */
export function TileGrid({
  children,
  label,
}: {
  children: ReactNode;
  /** Nome da lista para leitor de tela, quando não há título logo acima. */
  label?: string;
}) {
  return (
    <ul className={styles.grid} aria-label={label}>
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
