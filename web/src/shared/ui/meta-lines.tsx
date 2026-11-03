import type { ReactNode } from "react";

import styles from "./meta-lines.module.scss";

/**
 * O metadado de um tile (`LinkTile`, `ActionTile`) em mais de uma linha, para
 * ler de relance e para a quebra no celular não cortar um fato no meio. Item
 * vazio não ocupa linha.
 */
export function MetaLines({ lines }: { lines: ReactNode[] }) {
  return lines.filter(Boolean).map((line, index) => (
    // biome-ignore lint/suspicious/noArrayIndexKey: linhas fixas, na ordem.
    <span key={index} className={styles.line}>
      {line}
    </span>
  ));
}
