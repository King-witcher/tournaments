import { Children, isValidElement, type ReactNode } from "react";

import styles from "./task-notice-list.module.scss";

/**
 * Tarefas seguidas (`TaskNotice`), uma por linha, numa região nomeada para o
 * leitor de tela: sem título visível, porque cada aviso já diz o que falta.
 */
export function TaskNoticeList({
  label,
  children,
}: {
  /** Nome da região para o leitor de tela. */
  label: string;
  children: ReactNode;
}) {
  return (
    <section aria-label={label}>
      <ul className={styles.list}>
        {Children.toArray(children)
          .filter(isValidElement)
          .map((child) => (
            <li key={child.key}>{child}</li>
          ))}
      </ul>
    </section>
  );
}
