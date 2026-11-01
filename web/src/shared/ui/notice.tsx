import { Callout, InlineNotification, Stack } from "@carbon/react";
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
 * Aviso na página, sem botão de fechar. Informação e cuidado (`info`,
 * `warning`: o motivo de um campo travado, antes dele; a exclusão agendada)
 * carregam com a página e ficam no lugar: `Callout` do Carbon, que não
 * interrompe o leitor de tela. Erro e sucesso são o retorno de algo que
 * aconteceu: `InlineNotification`, lido na hora (`role="alert"` no erro,
 * `status` no sucesso). `action`: o que a pessoa pode fazer a respeito, logo
 * abaixo do aviso (o `InlineNotification` recusa link ou botão dentro dele,
 * e `children` também não leva). Serve em Server e em Client Component.
 */
export function Notice({ kind = "info", title, action, children }: Props) {
  const t = useTranslations("Common.notice");
  // `subtitle` só aceita texto; o resto (negrito, quebra) vai como filho.
  const text = typeof children === "string" ? children : undefined;
  const notice =
    kind === "info" || kind === "warning" ? (
      <Callout
        kind={kind}
        className={styles.notice}
        lowContrast
        title={title}
        subtitle={text}
        statusIconDescription={t(kind)}
      >
        {text === undefined && children}
      </Callout>
    ) : (
      <InlineNotification
        kind={kind}
        className={styles.notice}
        role={kind === "error" ? "alert" : "status"}
        lowContrast
        hideCloseButton
        title={title}
        subtitle={text}
        statusIconDescription={t(kind)}
      >
        {text === undefined && children}
      </InlineNotification>
    );
  if (!action) return notice;
  return (
    <Stack gap={3}>
      {notice}
      <div>{action}</div>
    </Stack>
  );
}
