"use client";

import { useTranslations } from "next-intl";

import { MapThumbnail } from "@/features/maps/components/map-thumbnail";
import { useLiveMatch } from "../hooks/use-live-match";
import { useSideNames } from "../hooks/use-side-names";
import { SidePicks } from "./side-picks";
import styles from "./upcoming-games.module.scss";

/**
 * As partidas depois da da vez que o veto já decidiu em parte, uma linha
 * compacta cada: a miniatura do mapa, "Partida N" e o mapa e os campeões de
 * cada lado, pequenos. As que ainda não têm nada (votação, sorteio) não
 * aparecem.
 */
export function UpcomingGames({ after }: { after: number }) {
  const t = useTranslations("Match.games");
  const tVs = useTranslations("Match");
  const { detail, viewerId } = useLiveMatch();
  const names = useSideNames(detail);
  const veto = detail.veto;
  if (!veto) return null;
  const games = veto.games.filter(
    (g) =>
      g.game > after &&
      (g.map || g.pick_a || g.pick_b || g.team_picks.length > 0),
  );

  return games.map((g) => (
    <li key={g.game} className={styles.row}>
      <span className={styles.thumb}>
        {g.map ? (
          <MapThumbnail slug={g.map.slug} />
        ) : (
          <span aria-hidden className={styles.noMap} />
        )}
      </span>
      <span className={styles.title}>
        <span className={styles.number}>{t("game", { number: g.game })}</span>
        {g.map && <span className={styles.map}>{g.map.name}</span>}
      </span>
      {veto.champion_phase && (
        <span className={styles.picks}>
          <span className={styles.srOnly}>{names.a}</span>
          <SidePicks
            veto={veto}
            game={g}
            side="a"
            viewerId={viewerId}
            size="sm"
          />
          <span aria-hidden className={styles.vs}>
            {tVs("vs")}
          </span>
          <span className={styles.srOnly}>{names.b}</span>
          <SidePicks
            veto={veto}
            game={g}
            side="b"
            viewerId={viewerId}
            size="sm"
          />
        </span>
      )}
    </li>
  ));
}
