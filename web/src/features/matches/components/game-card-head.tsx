"use client";

import { useTranslations } from "next-intl";

import { MapThumbnail } from "@/features/maps/components/map-thumbnail";
import { useLiveMatch } from "../hooks/use-live-match";
import { useSideNames } from "../hooks/use-side-names";
import type { MatchTarget } from "../lib/match-target";
import styles from "./game-card.module.scss";
import { GameCardSide } from "./game-card-side";
import { VetoTurn } from "./veto-turn";
import { VetoUndo } from "./veto-undo";
import { VetoVoteBoard } from "./veto-vote-board";

type Props = { target: MatchTarget; number: number };

/**
 * O alto do cartão da partida da vez, com o veto mais novo que a tela tem,
 * como os dois lados de frente: o lado A, no meio a imagem do mapa, "Partida
 * N", o mapa e de onde ele veio (quem escolheu, o decisivo, a votação, o
 * sorteio), e o lado B, cada um com o campeão. Na votação, os mapas
 * oferecidos embaixo; depois, desfazer o último passo enquanto dá.
 */
export function GameCardHead({ target, number }: Props) {
  const t = useTranslations("Match.card");
  const tGames = useTranslations("Match.games");
  const { detail } = useLiveMatch();
  const names = useSideNames(detail);
  const veto = detail.veto;
  const game = veto?.games.find((g) => g.game === number);
  const map = game?.map;
  const voting =
    veto?.map_mode === "vote" && !map && veto.turn?.action === "vote";
  const origin = !map
    ? null
    : game?.picked_by
      ? t("pickedBy", { name: names[game.picked_by] })
      : veto?.map_mode === "veto"
        ? t("decider")
        : veto?.map_mode === "vote"
          ? t("voted")
          : veto?.map_mode === "random"
            ? t("drawn")
            : null;

  return (
    <div className={styles.head}>
      <div className={styles.versus}>
        <GameCardSide side="a" number={number} />
        <div className={styles.center}>
          {map && (
            <span className={styles.image}>
              <MapThumbnail slug={map.slug} />
            </span>
          )}
          <h3 className={styles.number}>{tGames("game", { number })}</h3>
          {map && <p className={styles.mapName}>{map.name}</p>}
          {origin && <p className={styles.origin}>{origin}</p>}
        </div>
        <GameCardSide side="b" number={number} />
      </div>
      {voting && <VetoTurn />}
      {voting && <VetoVoteBoard target={target} game={number} />}
      <VetoUndo target={target} />
    </div>
  );
}
