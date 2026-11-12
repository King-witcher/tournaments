import type { ReactNode } from "react";

import styles from "./app-side-nav.module.scss";

type Props = {
  /** O tipo da área, num rótulo pequeno ("Comunidade", "Torneio"). */
  kind: string;
  /** O nome dela; longo, quebra em até duas linhas. */
  name: string;
  /** Antes do nome (o logo). */
  logo?: ReactNode;
  /**
   * Embaixo do nome, em texto secundário: o estado da área ou o papel de
   * quem vê ("Em andamento", "Organizador"). Texto, não etiqueta: está em
   * toda página da área e quase sempre é o caso esperado.
   */
  detail?: ReactNode;
};

/**
 * O topo do `AppSideNav` (prop `header`), como no IBM Cloud: o tipo da área
 * num rótulo pequeno e o nome dela, com o logo e o detalhe opcionais.
 */
export function AppSideNavTitle({ kind, name, logo, detail }: Props) {
  return (
    <>
      <p className={styles.kind}>{kind}</p>
      <div className={styles.titleRow}>
        {logo}
        <p className={styles.name}>{name}</p>
      </div>
      {detail && <p className={styles.detail}>{detail}</p>}
    </>
  );
}
