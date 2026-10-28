import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";

type Props = { event: components["schemas"]["MatchEvent"] };

/**
 * Um evento do histórico: o mapa (quando é de um mapa), quem fez o quê, o
 * resultado que ficou e o comentário.
 */
export function MatchEventRow({ event }: Props) {
  const t = useTranslations("Match.history");
  const o = event.outcome;
  const kind =
    event.kind === "voided" && !event.actor
      ? t("kinds.voided_auto")
      : t(`kinds.${event.kind}`, { actor: event.actor?.nickname ?? "" });

  let outcome: string | null = null;
  if (o && event.game != null) {
    outcome = t("mapWinner", { winner: o.winner_nickname });
  } else if (o?.walkover) {
    outcome = t("walkover", { winner: o.winner_nickname });
  } else if (o) {
    outcome = t("score", {
      winner: o.winner_nickname,
      winnerMaps: o.winner_maps ?? 0,
      loserMaps: o.loser_maps ?? 0,
    });
  }
  const parts = [
    event.game != null ? t("game", { number: event.game }) : null,
    kind,
    outcome,
  ].filter(Boolean);

  return (
    <li className="flex flex-col gap-0.5 text-sm">
      <span className="text-xs opacity-60">
        <LocalDateTime value={event.at} />
      </span>
      <span>{parts.join(" · ")}</span>
      {event.comment && (
        <q className="text-sm italic opacity-80">{event.comment}</q>
      )}
    </li>
  );
}
