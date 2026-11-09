import type { ReactNode } from "react";

import { signedNumber } from "../lib/score";
import styles from "./standing-diff.module.scss";

type Props = {
  value: number;
  /** O que compõe o saldo: pró – contra, ou o recorde da série. */
  detail: ReactNode;
};

/**
 * Um saldo da classificação: o número com sinal em cima, que é o que desempata,
 * e, embaixo, em segundo plano, de onde ele vem.
 */
export function StandingDiff({ value, detail }: Props) {
  return (
    <span className={styles.diff}>
      <span className={styles.value}>{signedNumber(value)}</span>
      <span className={styles.detail}>{detail}</span>
    </span>
  );
}
