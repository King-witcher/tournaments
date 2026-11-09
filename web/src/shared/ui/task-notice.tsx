import type { ReactNode } from "react";

import { Notice } from "./notice";
import styles from "./task-notice.module.scss";

type Props = {
  kind: "warning" | "info";
  /** Sobre o que é a tarefa. */
  title: string;
  /** O contexto, embaixo do título (`MetaLines`, para mais de uma linha). */
  meta?: ReactNode;
  /** O que fazer: um `Button`, `ButtonLink` (`ghost`, `sm`, o botão do `Callout`) ou o de uma feature. */
  action: ReactNode;
};

/**
 * Uma tarefa de quem vê: um `Notice` (`Callout` do Carbon) com o que falta e o
 * botão que resolve. Várias seguidas: `TaskNoticeList`. Serve em Server e em
 * Client Component.
 */
export function TaskNotice({ kind, title, meta, action }: Props) {
  return (
    <Notice kind={kind} title={title} action={action} wide>
      {meta && <div className={styles.meta}>{meta}</div>}
    </Notice>
  );
}
