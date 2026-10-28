import { InlineNotification } from "@carbon/react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import styles from "./notice.module.scss";

type Props = {
  kind?: "error" | "warning" | "info" | "success";
  /** Resumo curto em negrito antes do texto. */
  title?: string;
  /** Um botão ou link depois do texto (`Link`, `Button kind="ghost"`). */
  action?: ReactNode;
  children?: ReactNode;
};

/**
 * Aviso na página (`InlineNotification` do Carbon, sem botão de fechar): erro
 * que ocupa a página, aviso, informação (o motivo de um campo travado, antes
 * dele). Erro é lido na hora pelo leitor de tela (`role="alert"`); o resto,
 * quando der (`status`). `action`: o que a pessoa pode fazer a respeito.
 * Serve em Server e em Client Component.
 */
export function Notice({ kind = "info", title, action, children }: Props) {
  const t = useTranslations("Common.notice");
  // `subtitle` só aceita texto; o resto (com link, negrito) vai como filho.
  const text = typeof children === "string" ? children : undefined;
  return (
    <InlineNotification
      kind={kind}
      role={kind === "error" ? "alert" : "status"}
      lowContrast
      hideCloseButton
      title={title}
      subtitle={text}
      statusIconDescription={t(kind)}
    >
      {text === undefined && children}
      {action && <span className={styles.action}>{action}</span>}
    </InlineNotification>
  );
}
