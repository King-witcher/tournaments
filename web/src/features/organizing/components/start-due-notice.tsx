import { useLocale, useTranslations } from "next-intl";

import { localizedName } from "@/features/tournaments/lib/localize";
import type { components } from "@/shared/api/schema";
import { ButtonLink } from "@/shared/ui/button-link";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { MetaLines } from "@/shared/ui/meta-lines";
import { TaskNotice } from "@/shared/ui/task-notice";
import type { Severity } from "../lib/severity";

/**
 * Um torneio cujo início marcado chegou sem ser iniciado: embaixo do título,
 * a comunidade e o torneio, e o início marcado com quantos entrariam na
 * chave; o botão leva ao torneio (onde se inicia ou, nas Configurações, se
 * muda a data).
 */
export function StartDueNotice({
  kind,
  item,
  community,
}: {
  kind: Severity;
  item: components["schemas"]["StartDue"];
  /** O nome da comunidade, quando a lista é de várias. */
  community?: string;
}) {
  const t = useTranslations("Organizing.startDue");
  const locale = useLocale();
  const { tournament } = item;

  return (
    <TaskNotice
      kind={kind}
      title={t("title")}
      meta={
        <MetaLines
          lines={[
            `${community ? `${community} · ` : ""}${localizedName(tournament, locale)}`,
            <>
              {t.rich("startedAt", {
                date: () => <LocalDateTime value={item.starts_at} />,
              })}
              {" · "}
              {!item.has_check_in
                ? t("registered", { registered: item.registered })
                : item.status === "check_in"
                  ? t("checkedIn", {
                      checkedIn: item.checked_in,
                      registered: item.registered,
                    })
                  : t("checkInNotOpen", { registered: item.registered })}
            </>,
          ]}
        />
      }
      action={
        <ButtonLink
          href={`/${tournament.tenant_slug}/${tournament.slug}`}
          kind="ghost"
          size="sm"
        >
          {t("action")}
        </ButtonLink>
      }
    />
  );
}
