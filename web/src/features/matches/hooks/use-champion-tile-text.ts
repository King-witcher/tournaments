import { useFormatter, useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import type { ChampionTile } from "../lib/champion-tiles";

/**
 * Os textos de um campeão na grade do draft: `note`, a etiqueta curta à vista
 * ("Partida 1", "Partidas 1 e 2", "Usado na fase"), e `described`, o estado
 * por extenso para o leitor de tela ("banido por Ana"). `rule` é a regra de
 * não repetição do torneio (o motivo diz o escopo dela).
 */
export function useChampionTileText(
  names: { a: string; b: string },
  rule: components["schemas"]["NoRepeatRule"] | null,
) {
  const t = useTranslations("Match.veto.champions");
  const tGame = useTranslations("Match.games");
  const format = useFormatter();
  const repeat = rule ? rule.scope : "series";
  const games = (tile: ChampionTile) => ({
    count: tile.games.length,
    list: format.list(tile.games.map(String)),
  });

  const note = (tile: ChampionTile) =>
    tile.state === "picked"
      ? tGame("games", games(tile))
      : tile.reason === "repeat"
        ? t(`short.repeat.${repeat}`)
        : tile.reason
          ? t(`short.${tile.reason}`)
          : undefined;
  const described = (tile: ChampionTile) =>
    tile.state === "banned" && tile.side
      ? t("reasons.banned", { name: names[tile.side] })
      : tile.state === "picked" && tile.side
        ? t("reasons.picked", { name: names[tile.side], ...games(tile) })
        : tile.reason === "repeat"
          ? t(`reasons.repeat.${repeat}`)
          : tile.reason
            ? t(`reasons.${tile.reason}`)
            : undefined;

  return { note, described };
}
