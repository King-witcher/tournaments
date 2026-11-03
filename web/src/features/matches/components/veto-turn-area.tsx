import type { ReactNode } from "react";

import { VetoTurn } from "./veto-turn";
import styles from "./veto-turn-area.module.scss";

type Props = {
  /** Sem vez à vista (o passo feito, só para ver como terminou): só o conteúdo. */
  turn?: boolean;
  children: ReactNode;
};

/**
 * A vez e o que ela decide (o tabuleiro), no mesmo bloco: a barra da vez fica
 * presa ao rolar enquanto o bloco está à vista. Dentro do `Stack` da `Section`
 * ela não prenderia: cada filho de uma grade só anda na própria linha.
 */
export function VetoTurnArea({ turn = true, children }: Props) {
  return (
    <div className={styles.area}>
      {turn && <VetoTurn />}
      {children}
    </div>
  );
}
