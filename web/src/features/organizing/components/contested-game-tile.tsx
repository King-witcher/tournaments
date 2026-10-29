import { useLocale, useTranslations } from "next-intl";

import { useMatchLabels } from "@/features/matches/hooks/use-match-labels";
import { localizedName } from "@/features/tournaments/lib/localize";
import type { components } from "@/shared/api/schema";
import { LinkTile } from "@/shared/ui/link-tile";
import { StatusTag } from "@/shared/ui/status-tag";
import { TimeAgo } from "@/shared/ui/time-ago";
import { PendingMeta } from "./pending-meta";

/**
 * Um mapa contestado esperando a organização, como tile que leva à página
 * da partida (onde se resolve): o torneio, a comunidade, a partida e o
 * mapa, e quem contestou há quanto tempo.
 */
export function ContestedGameTile({
  item,
}: {
  item: components["schemas"]["ContestedGame"];
}) {
  const t = useTranslations("Organizing.contested");
  const locale = useLocale();
  const { code } = useMatchLabels(item.single);
  const { tournament } = item;
  const where = { match: code(item.key), game: item.game };
  const ago = () => <TimeAgo value={item.contested_at} />;

  return (
    <LinkTile
      href={`/${tournament.tenant_slug}/${tournament.slug}/matches/${item.key}`}
      title={localizedName(tournament, locale)}
      meta={
        <PendingMeta
          lines={[
            tournament.tenant_name,
            item.map
              ? t("whereMap", { ...where, map: item.map })
              : t("where", where),
            item.contested_by
              ? t.rich("by", { player: item.contested_by, ago })
              : t.rich("byUnknown", { ago }),
          ]}
        />
      }
      tags={<StatusTag status="contested">{t("tag")}</StatusTag>}
    />
  );
}
