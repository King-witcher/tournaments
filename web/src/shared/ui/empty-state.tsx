import type { ReactNode } from "react";

import styles from "./empty-state.module.scss";

type Props = {
  /** O que falta e o próximo passo ("Nenhum convite ativo. Gere um link…"). */
  children: ReactNode;
  /** A ação que resolve (um `ButtonLink` ou `Button`). */
  action?: ReactNode;
};

/**
 * Lista sem nada: diz o próximo passo, não só "nada aqui", com a ação ao
 * lado quando quem vê pode agir.
 */
export function EmptyState({ children, action }: Props) {
  return (
    <div className={styles.empty}>
      <p className={styles.text}>{children}</p>
      {action}
    </div>
  );
}
