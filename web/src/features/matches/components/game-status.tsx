import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import { LocalTime } from "@/shared/ui/local-time";
import { StatusTag } from "@/shared/ui/status-tag";
import { GAME_STATUS } from "../lib/status";
import { otherSide } from "../lib/veto";
import { DeadlineCountdown } from "./deadline-countdown";
import styles from "./game-status.module.scss";

type Props = {
  game: components["schemas"]["GameDetail"];
  names: { a: string; b: string };
  /** A célula da grade da linha. */
  className?: string;
};

/**
 * O estado de uma partida enviada: a etiqueta e, só quando diz algo, a frase
 * (até quando confirma sozinha, contestada por quem e o comentário, até
 * quando quem vê ainda contesta a confirmada).
 */
export function GameStatus({ game: g, names, className }: Props) {
  const t = useTranslations("Match.games");
  const deadline = g.confirm_by;
  let phrase: ReactNode = null;
  if (g.status === "pending" && g.reported_side && deadline) {
    phrase = t.rich("pending", {
      at: () => <LocalTime value={deadline} />,
      time: () => <DeadlineCountdown until={deadline} />,
    });
  } else if (g.status === "contested" && g.reported_side) {
    phrase = t("contested", {
      name: g.contested_by ?? names[otherSide(g.reported_side)],
      comment: g.contest_comment ?? "",
    });
  } else if (g.confirmed_via && g.can_contest) {
    phrase = `${t(`confirmed.${g.confirmed_via}`)} ${t("lateContest", names)}`;
  }

  return (
    <span className={cn(styles.status, className)}>
      <StatusTag status={GAME_STATUS[g.status]} size="sm">
        {t(`badge.${g.status}`)}
      </StatusTag>
      {phrase && <span className={styles.phrase}>{phrase}</span>}
    </span>
  );
}
