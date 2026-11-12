import { useLocale, useTranslations } from "next-intl";

import { useMatchLabels } from "@/features/matches/hooks/use-match-labels";
import { useTaskNoticeText } from "@/features/matches/hooks/use-task-notice-text";
import { localizedName } from "@/features/tournaments/lib/localize";
import type { components } from "@/shared/api/schema";
import { ButtonLink } from "@/shared/ui/button-link";
import { MetaLines } from "@/shared/ui/meta-lines";
import { TaskNotice } from "@/shared/ui/task-notice";
import { TimeAgo } from "@/shared/ui/time-ago";
import type { Severity } from "../lib/severity";

/**
 * Uma partida contestada esperando a organização: quem contestou o envio de
 * quem, no título; embaixo, a comunidade e o torneio, a série, a partida e o
 * mapa, com há quanto tempo; o botão leva à página da série, onde se resolve.
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
  const tPending = useTranslations("Bracket.pending");
  const locale = useLocale();
  const { code } = useMatchLabels(item.single);
  const { title } = useTaskNoticeText();
  const { tournament } = item;
  const where = { match: code(item.key), game: item.game };

  return (
    <TaskNotice
      kind={kind}
      title={title("resolve", {
        game: item.game,
        contested_by: item.contested_by,
        reported_by: item.reported_by,
        opponent_ready: false,
      })}
      meta={
        <MetaLines
          lines={[
            `${community ? `${community} · ` : ""}${localizedName(tournament, locale)}`,
            <>
              {item.map
                ? t("whereMap", { ...where, map: item.map })
                : t("where", where)}
              {" · "}
              <TimeAgo value={item.contested_at} />
            </>,
          ]}
        />
      }
      action={
        <ButtonLink
          href={`/${tournament.tenant_slug}/${tournament.slug}/matches/${item.key}`}
          kind="ghost"
          size="sm"
        >
          {tPending("open")}
        </ButtonLink>
      }
    />
  );
}
