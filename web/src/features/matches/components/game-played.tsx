import { Layer, Tile } from "@carbon/react";
import { useTranslations } from "next-intl";

import { GameScreenshots } from "@/features/screenshots/components/game-screenshots";
import type { components } from "@/shared/api/schema";
import { gameRowId, type MatchTarget } from "../lib/match-target";
import { GameAcceptContest } from "./game-accept-contest";
import styles from "./game-card.module.scss";
import { GameFrags } from "./game-frags";
import { GameOrganizerActions } from "./game-organizer-actions";
import { GameOutcome } from "./game-outcome";
import playedStyles from "./game-played.module.scss";
import { GameRespond } from "./game-respond";
import { GameVersus } from "./game-versus";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  game: Schemas["GameDetail"];
  names: { a: string; b: string };
  /** Os mapas que cabem na correção desta partida. */
  maps: Schemas["Map"][];
  /** O mapa que o veto deu a esta partida. */
  fixedMap?: Schemas["MapRef"];
  canManage: boolean;
  /** É a última partida enviada da série (a única que se anula). */
  last: boolean;
};

/**
 * Uma partida enviada, no mesmo cartão da da vez: o alto (os lados, o mapa
 * e os campeões), os frags embaixo de cada lado, o que ela diz (quem venceu,
 * o estado), o que quem vê pode fazer nela (confirmar, contestar, aceitar a
 * contestação, as ações da organização; ao colega que não pode, que é o capitão
 * quem responde) e os screenshots.
 */
export function GamePlayed({
  target,
  game: g,
  names,
  maps,
  fixedMap,
  canManage,
  last,
}: Props) {
  const t = useTranslations("Match.games");
  const responds = g.task === "respond";
  const contests = responds || g.can_contest;
  const accepts = g.task === "accept_contest";
  return (
    <Tile>
      <Layer className={styles.content}>
        <GameVersus number={g.number} map={g.map} />
        {g.map && <GameFrags game={g} names={names} />}
        <GameOutcome game={g} names={names} inlineFrags={!g.map} />
        {g.task === "await_captain" && <p>{t("captainResponds")}</p>}
        {(contests || accepts || canManage) && (
          // Uma linha só: as de jogador e as da organização, com o raro no ⋯.
          <div className={playedStyles.actions}>
            {contests && (
              <GameRespond
                target={target}
                number={g.number}
                canConfirm={responds}
              />
            )}
            {accepts && (
              <GameAcceptContest
                target={target}
                game={g}
                names={names}
                maps={maps}
                fixedMap={fixedMap}
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
        />
      </Layer>
    </Tile>
  );
}
