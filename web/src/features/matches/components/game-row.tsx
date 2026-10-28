import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Badge } from "@/shared/ui/badge";
import { LocalTime } from "@/shared/ui/local-time";
import { gameRowId, type MatchTarget } from "../lib/match-target";
import { DeadlineCountdown } from "./deadline-countdown";
import { GameOrganizerActions } from "./game-organizer-actions";
import { GameRespond } from "./game-respond";

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

const BADGE = {
  pending: "neutral",
  contested: "warning",
  confirmed: "success",
} as const;

/**
 * Um mapa enviado: número, mapa, quem venceu e os frags (sem confirmar, como
 * o que quem enviou diz), o estado (até quando confirma sozinho, contestado e
 * por quê, ou confirmado) e o que quem vê pode fazer nele.
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

  let status: React.ReactNode = null;
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

  return (
    // tabIndex -1: recebe o foco depois de confirmar, contestar ou resolver o
    // mapa, quando o botão usado some.
    <li
      id={gameRowId(g.number)}
      tabIndex={-1}
      className="flex flex-col gap-2 py-3"
    >
      {/* Número e mapa à esquerda, resultado à direita: dá para correr o olho
          pela coluna. Quebra em duas linhas quando não cabe (390px). */}
      <div className="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1">
        <span className="flex items-baseline gap-2">
          <span className="font-medium">
            {t("number", { number: g.number })}
          </span>
          <span className={g.map ? "" : "text-sm italic text-muted"}>
            {g.map?.name ?? t("noMap")}
          </span>
        </span>
        <span className="flex items-baseline gap-2">
          {/* Negrito só no que vale: sem confirmar, é a palavra de quem
              enviou, e o placar no topo ainda não conta o mapa. */}
          <span className={settled ? "font-semibold" : undefined}>
            {settled || !g.reported_side
              ? t("won", { name: winner })
              : t("claim", { reporter: names[g.reported_side], winner })}
          </span>
          {g.a_frags != null && g.b_frags != null && (
            <span className="text-sm tabular-nums text-muted">
              {g.a_frags} × {g.b_frags}
            </span>
          )}
        </span>
      </div>
      <div className="flex flex-wrap items-center gap-2 text-sm">
        <Badge variant={BADGE[g.status]}>{t(`badge.${g.status}`)}</Badge>
        {status && <span className="text-muted">{status}</span>}
      </div>
      {g.task === "respond" && (
        <GameRespond target={target} number={g.number} />
      )}
      {canManage && (
        <GameOrganizerActions
          target={target}
          game={g}
          names={names}
          maps={maps}
          responds={g.task === "respond"}
          last={last}
        />
      )}
    </li>
  );
}
