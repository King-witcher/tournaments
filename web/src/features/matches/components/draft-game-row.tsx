import { useTranslations } from "next-intl";

import { MapThumbnail } from "@/features/maps/components/map-thumbnail";
import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import styles from "./draft-summary.module.scss";
import { SidePicks } from "./side-picks";

type Schemas = components["schemas"];

type Props = {
  veto: Schemas["VetoView"];
  game: Schemas["VetoGame"];
  viewerId: number | null;
  /** A partida da vez no draft: em destaque. */
  current: boolean;
};

/** Uma partida no resumo do draft: [campeão de A] mapa [campeão de B]. */
export function DraftGameRow({ veto, game, viewerId, current }: Props) {
  const t = useTranslations("Match.veto.draft");
  return (
    <tr className={cn(current && styles.current)}>
      <td>
        <SidePicks veto={veto} game={game} side="a" viewerId={viewerId} />
      </td>
      <th scope="row" className={styles.middle}>
        <span className={styles.map}>
          {game.map ? (
            <MapThumbnail slug={game.map.slug} />
          ) : (
            <span aria-hidden className={styles.noMap} />
          )}
        </span>
        <span className={styles.game}>{t("game", { game: game.game })}</span>
        {game.map && <span className={styles.mapName}>{game.map.name}</span>}
      </th>
      <td>
        <SidePicks veto={veto} game={game} side="b" viewerId={viewerId} />
      </td>
    </tr>
  );
}
