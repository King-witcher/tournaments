import type { ReactNode } from "react";

import { Link } from "@/i18n/navigation";
import styles from "./row-link.module.scss";

type Props = {
  href: string;
  /** O nome do item, na cor de link. */
  title: ReactNode;
  /** Metadado (status, data, papel), em texto secundário. */
  children?: ReactNode;
};

/**
 * @deprecated Lista grande: `DataTable` com `href` na linha. Lista "minha":
 * `TileGrid` + `LinkTile`. Este é a linha antiga, inteira clicável.
 */
export function RowLink({ href, title, children }: Props) {
  return (
    <Link href={href} className={styles.row}>
      <span className={styles.title}>{title}</span>
      {children && <span className={styles.meta}>{children}</span>}
    </Link>
  );
}
