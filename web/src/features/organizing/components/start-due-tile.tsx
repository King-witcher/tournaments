import { useLocale, useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { localizedName } from "@/features/tournaments/lib/localize";
import type { components } from "@/shared/api/schema";
import { ActionTile } from "@/shared/ui/action-tile";
import { ButtonLink } from "@/shared/ui/button-link";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { MetaLines } from "@/shared/ui/meta-lines";

/**
 * Um torneio cujo início marcado chegou sem ser iniciado: a comunidade, o
 * início marcado, quantos entrariam na chave e o botão que leva ao torneio
 * (onde se inicia ou, nas Configurações, se muda a data).
 */
export function StartDueTile({
  item,
  avatar,
}: {
  item: components["schemas"]["StartDue"];
  /** O logo da comunidade, quando a lista leva. */
  avatar: ReactNode;
}) {
  const t = useTranslations("Organizing.startDue");
  const locale = useLocale();
  const { tournament } = item;

  return (
    <ActionTile
      avatar={avatar}
      title={localizedName(tournament, locale)}
      meta={
        <MetaLines
          lines={[
            <>
              {tournament.tenant_name}
              {" · "}
              {t.rich("startedAt", {
                date: () => <LocalDateTime value={item.starts_at} />,
              })}
            </>,
            item.has_check_in
              ? t("checkedIn", {
                  checkedIn: item.checked_in,
                  registered: item.registered,
                })
              : t("registered", { registered: item.registered }),
          ]}
        />
      }
      action={
        <ButtonLink
          href={`/${tournament.tenant_slug}/${tournament.slug}`}
          kind="tertiary"
          size="sm"
        >
          {t("action")}
        </ButtonLink>
      }
    />
  );
}
