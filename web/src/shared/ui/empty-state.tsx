import type { ReactNode } from "react";

import styles from "./empty-state.module.scss";

type Props = {
  /** O estado, curto ("Nenhum link de inscrição."), sem ensinar o uso. */
  children: ReactNode;
  /** A ação que resolve (um `ButtonLink` ou `Button`). */
  action?: ReactNode;
};

/**
 * Lista sem nada: o estado e, quando quem vê pode agir, a ação ao lado. O
 * botão já diz o próximo passo; o texto não o repete.
 */
export function EmptyState({ children, action }: Props) {
  return (
    <div className={styles.empty}>
      <p className={styles.text}>{children}</p>
      {action}
    </div>
  );
}
