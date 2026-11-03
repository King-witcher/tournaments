import { useLocale, useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { useMatchLabels } from "@/features/matches/hooks/use-match-labels";
import { localizedName } from "@/features/tournaments/lib/localize";
import type { components } from "@/shared/api/schema";
import { ActionTile } from "@/shared/ui/action-tile";
import { ButtonLink } from "@/shared/ui/button-link";
import { MetaLines } from "@/shared/ui/meta-lines";
import { TimeAgo } from "@/shared/ui/time-ago";

/**
 * Um mapa contestado esperando a organização: o torneio, a comunidade, a
 * partida e o mapa, quem contestou há quanto tempo e o botão que leva à página
 * da partida, onde se resolve.
 */
export function ContestedGameTile({
  item,
  avatar,
}: {
  item: components["schemas"]["ContestedGame"];
  /** O logo da comunidade, quando a lista leva. */
  avatar: ReactNode;
}) {
  const t = useTranslations("Organizing.contested");
  const locale = useLocale();
  const { code } = useMatchLabels(item.single);
  const { tournament } = item;
  const where = { match: code(item.key), game: item.game };
  const ago = () => <TimeAgo value={item.contested_at} />;

  return (
    <ActionTile
      avatar={avatar}
      title={localizedName(tournament, locale)}
      meta={
        <MetaLines
          lines={[
            `${tournament.tenant_name} · ${
              item.map
                ? t("whereMap", { ...where, map: item.map })
                : t("where", where)
            }`,
            item.contested_by
              ? t.rich("by", { player: item.contested_by, ago })
              : t.rich("byUnknown", { ago }),
          ]}
        />
      }
      action={
        <ButtonLink
          href={`/${tournament.tenant_slug}/${tournament.slug}/matches/${item.key}`}
          kind="tertiary"
          size="sm"
        >
          {t("action")}
        </ButtonLink>
      }
    />
  );
}
