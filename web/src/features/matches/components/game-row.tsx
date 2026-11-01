import { useTranslations } from "next-intl";

import { MapThumbnail } from "@/features/maps/components/map-thumbnail";
import { GameScreenshots } from "@/features/screenshots/components/game-screenshots";
import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import { gameRowId, type MatchTarget } from "../lib/match-target";
import { GameOrganizerActions } from "./game-organizer-actions";
import { GameRespond } from "./game-respond";
import { GameResult } from "./game-result";
import styles from "./game-row.module.scss";
import { GameStatus } from "./game-status";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  game: Schemas["GameDetail"];
  names: { a: string; b: string };
  /** Os mapas que cabem na correção desta partida. */
  maps: Schemas["Map"][];
  canManage: boolean;
  /** É a última partida da série. */
  last: boolean;
};

/**
 * Uma partida enviada, numa linha compacta: a miniatura do mapa, "Partida N"
 * e o mapa, o resultado, o estado e, embaixo, o que quem vê pode fazer nela
 * (confirmar, contestar, as ações da organização) e os screenshots.
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
  const responds = g.task === "respond";
  const contests = responds || g.can_contest;

  return (
    // tabIndex -1: recebe o foco depois de confirmar, contestar ou resolver a
    // partida, quando o botão usado some.
    <li id={gameRowId(g.number)} tabIndex={-1} className={styles.row}>
      <span className={styles.thumb}>
        <MapThumbnail slug={g.map?.slug ?? ""} />
      </span>
      <span className={styles.title}>
        <span className={styles.number}>{t("game", { number: g.number })}</span>
        <span className={cn(styles.map, !g.map && styles.missing)}>
          {g.map?.name ?? t("noMap")}
        </span>
      </span>
      <GameResult game={g} names={names} className={styles.result} />
      <GameStatus game={g} names={names} className={styles.status} />
      {(contests || canManage) && (
        <div className={styles.actions}>
          {contests && (
            <GameRespond
              target={target}
              number={g.number}
              canConfirm={responds}
            />
          )}
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
      <GameScreenshots
        target={target}
        number={g.number}
        focusId={gameRowId(g.number)}
        className={styles.screenshots}
      />
    </li>
  );
}
