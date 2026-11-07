import { getTranslations } from "next-intl/server";

import { Link } from "@/shared/ui/link";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { Notice } from "@/shared/ui/notice";

type Props = {
  slug: string;
  /** Quando a comunidade vai ser excluída (ISO). */
  date: string;
  /** Leva ao "Cancelar exclusão" das Configurações (o dono, fora delas). */
  cancel: boolean;
};

/**
 * A comunidade vai ser excluída e, até lá, é só leitura: o aviso em toda
 * seção dela. Para o dono, fora das Configurações, o link até onde ele
 * cancela.
 */
export async function TenantDeletionNotice({ slug, date, cancel }: Props) {
  const t = await getTranslations("Tenant.deletionNotice");
  return (
    <Notice
      kind="warning"
      title={t("title")}
      action={
        cancel && <Link href={`/${slug}/settings#deletion`}>{t("cancel")}</Link>
      }
    >
      {/* Num span: o conteúdo do aviso é flex, e a data quebraria a frase. */}
      <span>
        {t.rich("text", { date: () => <LocalDateTime value={date} /> })}
      </span>
    </Notice>
  );
}
