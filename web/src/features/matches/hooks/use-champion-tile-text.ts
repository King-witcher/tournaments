import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import type { ChampionTile } from "../lib/champion-tiles";

/**
 * Os textos de um campeão na grade do draft: `hint`, o motivo curto de quem vê
 * não poder escolhê-lo ("Escolhido pelo colega", "Usado na fase"), na dica do
 * tile, e `described`, o banido por extenso para o leitor de tela ("banido por
 * Ana"). `rule` é a regra de não repetição do torneio (o motivo diz o escopo
 * dela).
 */
export function useChampionTileText(
  names: { a: string; b: string },
  rule: components["schemas"]["NoRepeatRule"] | null,
) {
  const t = useTranslations("Match.veto.champions");
  const repeat = rule ? rule.scope : "series";

  const hint = (tile: ChampionTile) =>
    tile.reason === "repeat"
      ? t(`short.repeat.${repeat}`)
      : tile.reason
        ? t(`short.${tile.reason}`)
        : undefined;
  const described = (tile: ChampionTile) =>
    tile.state === "banned" && tile.side
      ? t("reasons.banned", { name: names[tile.side] })
      : undefined;

  return { hint, described };
}
