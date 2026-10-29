import { useTranslations } from "next-intl";

import { shownNickname } from "@/features/members/lib/nickname";
import { useTeamName } from "@/features/teams/hooks/use-team-name";
import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import styles from "./match-event-row.module.scss";

type Props = { event: components["schemas"]["MatchEvent"] };

/**
 * Um evento do histórico: quando, o mapa (quando é de um mapa), quem fez o
 * quê, o resultado que ficou e o comentário.
 */
export function MatchEventRow({ event }: Props) {
  const t = useTranslations("Match.history");
  const tMember = useTranslations("Member");
  const teamName = useTeamName();
  const o = event.outcome;
  const actor = shownNickname(
    event.actor?.nickname ?? "",
    tMember("deletedAccount"),
  );
  const kind =
    event.kind === "voided" && !event.actor
      ? t("kinds.voided_auto")
      : t(`kinds.${event.kind}`, { actor: event.actor ? actor : "" });

  let outcome: string | null = null;
  const winner = o ? teamName(o.winner) : "";
  if (o && event.game != null) {
    outcome = t("mapWinner", { winner });
  } else if (o?.walkover) {
    outcome = t("walkover", { winner });
  } else if (o) {
    outcome = t("score", {
      winner,
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
    <li className={styles.event}>
      <span className={styles.time}>
        <LocalDateTime value={event.at} />
      </span>
      <span>{parts.join(" · ")}</span>
      {event.comment && <q className={styles.comment}>{event.comment}</q>}
    </li>
  );
}
