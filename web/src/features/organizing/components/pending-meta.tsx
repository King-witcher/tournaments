import type { ReactNode } from "react";

import styles from "./pending-meta.module.scss";

/**
 * O metadado de um tile de pendência (`LinkTile.meta`), um fato por linha:
 * a comunidade, onde e o que espera.
 */
export function PendingMeta({ lines }: { lines: ReactNode[] }) {
  return lines.map((line, index) => (
    // biome-ignore lint/suspicious/noArrayIndexKey: linhas fixas, na ordem.
    <span key={index} className={styles.line}>
      {line}
    </span>
  ));
}
