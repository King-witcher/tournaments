import { InlineNotification, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

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
 * quando der (`status`). `action`: o que a pessoa pode fazer a respeito,
 * logo abaixo do aviso: o Carbon recusa link ou botão dentro dele (lança erro
 * em dev). Pelo mesmo motivo, `children` não leva link.
 * Serve em Server e em Client Component.
 */
export function Notice({ kind = "info", title, action, children }: Props) {
  const t = useTranslations("Common.notice");
  // `subtitle` só aceita texto; o resto (negrito, quebra) vai como filho.
  const text = typeof children === "string" ? children : undefined;
  const notice = (
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
  if (!action) return notice;
  return (
    <Stack gap={3}>
      {notice}
      <div>{action}</div>
    </Stack>
  );
}
