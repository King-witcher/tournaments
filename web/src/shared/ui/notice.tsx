import { InlineNotification } from "@carbon/react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

type Props = {
  kind?: "error" | "warning" | "info" | "success";
  /** Resumo curto em negrito antes do texto. */
  title?: string;
  children: ReactNode;
};

/**
 * Aviso na página (`InlineNotification` do Carbon, sem botão de fechar): erro
 * que ocupa a página, aviso, informação (o motivo de um campo travado, antes
 * dele). Erro é lido na hora pelo leitor de tela (`role="alert"`); o resto,
 * quando der (`status`). Serve em Server e em Client Component.
 */
export function Notice({ kind = "info", title, children }: Props) {
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
    </InlineNotification>
  );
}
