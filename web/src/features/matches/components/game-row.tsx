import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { GamePrints } from "@/features/prints/components/game-prints";
import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import { LocalTime } from "@/shared/ui/local-time";
import { StatusTag } from "@/shared/ui/status-tag";
import { gameRowId, type MatchTarget } from "../lib/match-target";
import { GAME_STATUS } from "../lib/status";
import { DeadlineCountdown } from "./deadline-countdown";
import { GameOrganizerActions } from "./game-organizer-actions";
import { GameRespond } from "./game-respond";
import styles from "./game-row.module.scss";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  game: Schemas["GameDetail"];
  names: { a: string; b: string };
  maps: Schemas["Map"][];
  canManage: boolean;
  /** É o último mapa da série. */
  last: boolean;
};

/**
 * Um mapa enviado: número, mapa, quem venceu e os frags (sem confirmar, como
 * o que quem enviou diz), a etiqueta e a frase do estado (até quando
 * confirma sozinho, contestado e por quê, ou confirmado e como) e, embaixo,
 * o que quem vê pode fazer nele.
 */
export function GameRow({
  target,
  game: g,
  names,
  maps,
  canManage,
  last,
}: Props) {
  const t = useTranslations("Match.games");
  const other = (side: "a" | "b") => (side === "a" ? "b" : "a");
  const deadline = g.confirm_by;
  const winner = names[g.winner];
  const settled = g.status === "confirmed";
  const hidden = (chunks: ReactNode) => (
    <span className={styles.srOnly}>{chunks}</span>
  );

  let status: ReactNode = null;
  if (g.status === "pending" && g.reported_side && deadline) {
    status = t.rich("pending", {
      at: () => <LocalTime value={deadline} />,
      time: () => <DeadlineCountdown until={deadline} />,
    });
  } else if (g.status === "contested" && g.reported_side) {
    status = t("contested", {
      name: names[other(g.reported_side)],
      comment: g.contest_comment ?? "",
    });
  } else if (g.confirmed_via) {
    status = t(`confirmed.${g.confirmed_via}`);
  }
  const responds = g.task === "respond";

  return (
    // tabIndex -1: recebe o foco depois de confirmar, contestar ou resolver o
    // mapa, quando o botão usado some.
    <li id={gameRowId(g.number)} tabIndex={-1} className={styles.row}>
      <span className={styles.num}>
        {t.rich("numberCell", {
          number: g.number,
          prefix: (chunks) => (
            <span className={styles.numPrefix}>{chunks}</span>
          ),
        })}
      </span>
      <span className={cn(styles.map, !g.map && styles.missing)}>
        {g.map?.name ?? t("noMap")}
      </span>
      {/* Negrito só no que vale: sem confirmar, é a palavra de quem enviou,
          e o placar no topo ainda não conta o mapa. */}
      <span className={cn(styles.result, settled && styles.settled)}>
        {settled || !g.reported_side
          ? t("won", { name: winner })
          : t("claim", { reporter: names[g.reported_side], winner })}
      </span>
      <span className={styles.frags}>
        {g.a_frags != null &&
          g.b_frags != null &&
          t.rich("fragsCell", { a: g.a_frags, b: g.b_frags, prefix: hidden })}
      </span>
      <span className={styles.status}>
        <StatusTag status={GAME_STATUS[g.status]} size="sm">
          {t(`badge.${g.status}`)}
        </StatusTag>
        {status && <span className={styles.phrase}>{status}</span>}
      </span>
      {(responds || canManage) && (
        <div className={styles.actions}>
          {responds && <GameRespond target={target} number={g.number} />}
          {canManage && (
            <GameOrganizerActions
              target={target}
              game={g}
              names={names}
              maps={maps}
              responds={responds}
              last={last}
            />
          )}
        </div>
      )}
      <GamePrints
        target={target}
        number={g.number}
        focusId={gameRowId(g.number)}
        className={styles.prints}
      />
    </li>
  );
}
