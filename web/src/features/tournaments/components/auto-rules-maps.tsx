import { useTranslations } from "next-intl";

import { MapGrid } from "@/features/maps/components/map-grid";
import { MapThumbnail } from "@/features/maps/components/map-thumbnail";
import type { components } from "@/shared/api/schema";
import { AutoRulesBlock } from "./auto-rules-block";
import styles from "./auto-rules-maps.module.scss";

type Props = {
  tournament: components["schemas"]["Tournament"];
  /** Os mapas ativos (`GET /maps`), para o nome de cada slug do pool. */
  maps: components["schemas"]["Map"][];
};

/**
 * Os mapas do pool do torneio, na ordem em que ele os configurou, só para
 * ler: a imagem e o nome de cada um. Sai quando o mapa é combinado fora
 * (modo desligado), porque não há pool.
 */
export function AutoRulesMaps({ tournament, maps }: Props) {
  const t = useTranslations("Tournament.autoRules.pool");
  const { map_mode: mapMode, map_pool: pool } = tournament.veto;
  if (mapMode === "off" || pool.length === 0) return null;
  const names = new Map(maps.map((m) => [m.slug, m.name]));
  return (
    <AutoRulesBlock title={t("title")}>
      <MapGrid as="ul">
        {pool.map((slug) => (
          <li key={slug} className={styles.map}>
            <MapThumbnail slug={slug} />
            <span className={styles.name}>{names.get(slug) ?? slug}</span>
          </li>
        ))}
      </MapGrid>
    </AutoRulesBlock>
  );
}
