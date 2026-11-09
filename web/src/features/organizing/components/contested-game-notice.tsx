import { useLocale, useTranslations } from "next-intl";

import { useMatchLabels } from "@/features/matches/hooks/use-match-labels";
import { localizedName } from "@/features/tournaments/lib/localize";
import type { components } from "@/shared/api/schema";
import { ButtonLink } from "@/shared/ui/button-link";
import { MetaLines } from "@/shared/ui/meta-lines";
import { TaskNotice } from "@/shared/ui/task-notice";
import { TimeAgo } from "@/shared/ui/time-ago";
import type { Severity } from "../lib/severity";

/**
 * Um mapa contestado esperando a organização: o torneio, a comunidade, a
 * partida e o mapa, quem contestou há quanto tempo e o botão que leva à página
 * da partida, onde se resolve.
 */
export function ContestedGameNotice({
  kind,
  item,
  community,
}: {
  kind: Severity;
  item: components["schemas"]["ContestedGame"];
  /** O nome da comunidade, quando a lista é de várias. */
  community?: string;
}) {
  const t = useTranslations("Organizing.contested");
  const locale = useLocale();
  const { code } = useMatchLabels(item.single);
  const { tournament } = item;
  const where = { match: code(item.key), game: item.game };
  const ago = () => <TimeAgo value={item.contested_at} />;

  return (
    <TaskNotice
      kind={kind}
      title={localizedName(tournament, locale)}
      meta={
        <MetaLines
          lines={[
            `${community ? `${community} · ` : ""}${
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
          kind="ghost"
          size="sm"
        >
          {t("action")}
        </ButtonLink>
      }
    />
  );
}
