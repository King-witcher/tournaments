import type { ReactNode } from "react";

import styles from "./map-grid.module.scss";

type Props = {
  /** `ul` quando cada filho é um `li` (a lista só para ler); `div` no resto. */
  as?: "div" | "ul";
  children: ReactNode;
};

/**
 * A grade das caixas de mapa (imagem 16:9 e nome): a do pool do formulário e
 * a da aba Regras. Os filhos ocupam uma coluna; um título de parte pode
 * ocupar a linha toda (`grid-column: 1 / -1`).
 */
export function MapGrid({ as: Tag = "div", children }: Props) {
  return <Tag className={styles.grid}>{children}</Tag>;
}
