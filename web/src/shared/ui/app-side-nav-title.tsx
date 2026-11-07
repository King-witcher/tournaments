import type { ReactNode } from "react";

import styles from "./app-side-nav.module.scss";

type Props = {
  /** O tipo da área, num rótulo pequeno ("Comunidade", "Torneio"). */
  kind: string;
  /** O nome dela; longo, quebra em até duas linhas. */
  name: string;
  /** Antes do nome (o logo). */
  logo?: ReactNode;
  /** Embaixo do nome (o estado, o papel de quem vê). */
  tags?: ReactNode;
};

/**
 * O topo do `AppSideNav` (prop `header`), como no IBM Cloud: o tipo da área
 * num rótulo pequeno e o nome dela, com o logo e as tags opcionais.
 */
export function AppSideNavTitle({ kind, name, logo, tags }: Props) {
  return (
    <>
      <p className={styles.kind}>{kind}</p>
      <div className={styles.titleRow}>
        {logo}
        <p className={styles.name}>{name}</p>
      </div>
      {tags && <div className={styles.tags}>{tags}</div>}
    </>
  );
}
