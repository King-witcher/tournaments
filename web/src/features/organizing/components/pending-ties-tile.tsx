import { useLocale, useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { localizedName } from "@/features/tournaments/lib/localize";
import type { components } from "@/shared/api/schema";
import { ActionTile } from "@/shared/ui/action-tile";
import { ButtonLink } from "@/shared/ui/button-link";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { MetaLines } from "@/shared/ui/meta-lines";

/**
 * Um torneio cujas séries acabaram com empate na classificação esperando a
 * organização: a comunidade, desde quando espera e o botão que leva à aba
 * Classificação, onde se decide a ordem.
 */
export function PendingTiesTile({
  item,
  avatar,
}: {
  item: components["schemas"]["PendingTies"];
  /** O logo da comunidade, quando a lista leva. */
  avatar: ReactNode;
}) {
  const t = useTranslations("Organizing.pendingTies");
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
              {t("title")}
            </>,
            t.rich("since", {
              date: () => <LocalDateTime value={item.since} />,
            }),
          ]}
        />
      }
      action={
        <ButtonLink
          href={`/${tournament.tenant_slug}/${tournament.slug}/standings`}
          kind="tertiary"
          size="sm"
        >
          {t("action")}
        </ButtonLink>
      }
    />
  );
}
