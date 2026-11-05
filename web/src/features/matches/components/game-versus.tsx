"use client";

import { useTranslations } from "next-intl";

import { MapThumbnail } from "@/features/maps/components/map-thumbnail";
import { isEvenSeries } from "@/features/tournaments/lib/format";
import type { components } from "@/shared/api/schema";
import { useLiveMatch } from "../hooks/use-live-match";
import { useSideNames } from "../hooks/use-side-names";
import { GameCardSide } from "./game-card-side";
import styles from "./game-versus.module.scss";

type Props = {
  /** A partida do cartão. */
  number: number;
  /** O mapa enviado (sem veto, ele não sai do veto). */
  map?: components["schemas"]["MapRef"] | null;
};

/**
 * O alto do cartão da partida, como os dois lados de frente: o lado A, no
 * meio a imagem do mapa, o nome e de onde ele veio (quem escolheu, o
 * decisivo, a votação, o sorteio; na série par, o veto só tem bans e não há o
 * que dizer), e o lado B, cada um com o campeão. Some enquanto a partida não tem mapa (votação, mapa combinado fora do site).
 */
export function GameVersus({ number, map: sent }: Props) {
  const t = useTranslations("Match.card");
  const { detail } = useLiveMatch();
  const names = useSideNames(detail);
  const veto = detail.veto;
  const game = veto?.games.find((g) => g.game === number);
  const map = game?.map ?? sent;
  if (!map) return null;
  const origin = game?.picked_by
    ? t("pickedBy", { name: names[game.picked_by] })
    : !game?.map
      ? null
      : veto?.map_mode === "veto"
        ? isEvenSeries(detail.summary.best_of)
          ? null
          : t("decider")
        : veto?.map_mode === "vote"
          ? t("voted")
          : veto?.map_mode === "random"
            ? t("drawn")
            : null;

  return (
    <div className={styles.versus}>
      <GameCardSide side="a" number={number} />
      <div className={styles.center}>
        <span className={styles.image}>
          <MapThumbnail slug={map.slug} />
        </span>
        <span className={styles.mapName}>{map.name}</span>
        {origin && <span className={styles.origin}>{origin}</span>}
      </div>
      <GameCardSide side="b" number={number} />
    </div>
  );
}
