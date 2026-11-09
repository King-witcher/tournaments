"use client";

import { type ReactNode, useId } from "react";

import { DescribedByContext } from "./described-by-context";
import { Notice } from "./notice";
import styles from "./task-notice.module.scss";

type Props = {
  kind: "warning" | "info";
  /** Sobre o que é a tarefa. */
  title: string;
  /** O contexto, menor, embaixo do título (`MetaLines`, para mais de uma linha). */
  meta?: ReactNode;
  /** O que fazer: um `Button`, `ButtonLink` (`ghost`, `sm`, o botão do `Callout`) ou o de uma feature feito com eles. */
  action: ReactNode;
};

/**
 * Uma tarefa de quem vê: um `Notice` (`Callout` do Carbon) com o que falta e,
 * dentro dele, o botão que resolve, à direita do texto (no celular, embaixo).
 * O `Callout` exige `aria-describedby` em quem fica dentro dele: `Button` e
 * `ButtonLink` já o recebem daqui, e controle de outro tipo precisa pô-lo à
 * mão. Várias seguidas: `TaskNoticeList`. Serve em Server e em Client
 * Component.
 */
export function TaskNotice({ kind, title, meta, action }: Props) {
  const titleId = useId();
  return (
    <Notice kind={kind} className={styles.notice} wide>
      <div className={styles.row}>
        <div className={styles.text}>
          <div id={titleId} className={styles.title}>
            {title}
          </div>
          {meta && <div className={styles.meta}>{meta}</div>}
        </div>
        <div className={styles.action}>
          <DescribedByContext value={titleId}>{action}</DescribedByContext>
        </div>
      </div>
    </Notice>
  );
}
