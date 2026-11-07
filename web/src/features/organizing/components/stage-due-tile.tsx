import { useLocale, useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { localizedName } from "@/features/tournaments/lib/localize";
import type { components } from "@/shared/api/schema";
import { ActionTile } from "@/shared/ui/action-tile";
import { ButtonLink } from "@/shared/ui/button-link";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { MetaLines } from "@/shared/ui/meta-lines";

/**
 * Um torneio com a fase seguinte pronta para iniciar (a anterior terminou):
 * a comunidade, a fase, desde quando espera e o botão que leva ao torneio,
 * onde a organização abre o check-in e inicia a fase.
 */
export function StageDueTile({
  item,
  avatar,
}: {
  item: components["schemas"]["StageDue"];
  /** O logo da comunidade, quando a lista leva. */
  avatar: ReactNode;
}) {
  const t = useTranslations("Organizing.stageDue");
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
              {t("title", { number: item.stage })}
            </>,
            t.rich("since", {
              date: () => <LocalDateTime value={item.since} />,
            }),
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
