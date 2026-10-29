import { getTranslations } from "next-intl/server";

import { LocalDateTime } from "@/shared/ui/local-date-time";
import { Notice } from "@/shared/ui/notice";

type Props = {
  /** `?deletion_scheduled=` da página de entrar: a data (ISO) da exclusão. */
  value: string | string[] | undefined;
};

/**
 * Na página de entrar, logo depois de pedir a exclusão da conta: a data e
 * que entrar de novo antes cancela (o formulário fica logo embaixo).
 */
export async function AccountDeletionScheduled({ value }: Props) {
  if (typeof value !== "string" || Number.isNaN(Date.parse(value))) {
    return null;
  }
  const t = await getTranslations("Login");

  return (
    <Notice kind="info" title={t("deletionScheduledTitle")}>
      {/* Num span: o conteúdo do aviso é flex, e a data quebraria a frase. */}
      <span>
        {t.rich("deletionScheduled", {
          date: () => <LocalDateTime value={value} />,
        })}
      </span>
    </Notice>
  );
}
