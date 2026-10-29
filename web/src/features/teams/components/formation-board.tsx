import type { ReactNode } from "react";

import { cn } from "@/shared/lib/cn";
import styles from "./formation-board.module.scss";

type Props = {
  /** A coluna dos times (`TeamsColumn`). */
  teams: ReactNode;
  /** A lista dos reservas (`ReservesBlock`). */
  reserves: ReactNode;
  /**
   * Na edição, no computador, os reservas acompanham a rolagem: dá para
   * arrastar um jogador do último time para a reserva sem rolar de volta.
   */
  sticky?: boolean;
};

/**
 * A formação em duas colunas no computador (os times à esquerda, os reservas
 * à direita) e numa só no celular, os reservas depois dos times.
 */
export function FormationBoard({ teams, reserves, sticky }: Props) {
  return (
    <div className={styles.board}>
      <div className={styles.main}>{teams}</div>
      <div className={cn(styles.side, sticky && styles.sticky)}>{reserves}</div>
    </div>
  );
}
