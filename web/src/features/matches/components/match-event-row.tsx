import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";

type Props = { event: components["schemas"]["MatchEvent"] };

/** Um evento do histórico: quem fez o quê, o placar que ficou e o comentário. */
export function MatchEventRow({ event }: Props) {
  const t = useTranslations("Match.history");
  const o = event.outcome;

  return (
    <li className="flex flex-col gap-0.5 text-sm">
      <span className="text-xs opacity-60">
        <LocalDateTime value={event.at} />
      </span>
      <span>
        {t(`kinds.${event.kind}`, { actor: event.actor?.nickname ?? "" })}
        {o &&
          ` · ${
            o.walkover
              ? t("walkover", { winner: o.winner_nickname })
              : t("score", {
                  winner: o.winner_nickname,
                  winnerMaps: o.winner_maps ?? 0,
                  loserMaps: o.loser_maps ?? 0,
                })
          }`}
      </span>
      {event.comment && (
        <q className="text-sm italic opacity-80">{event.comment}</q>
      )}
    </li>
  );
}
