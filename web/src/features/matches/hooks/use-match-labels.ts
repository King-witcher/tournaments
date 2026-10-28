import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { parseMatchKey, roundName } from "../lib/bracket-layout";

type Schemas = components["schemas"];

/**
 * Nomes das partidas e rodadas no idioma de quem vê: código curto
 * (`C1.2`), título da rodada, de onde vem um lado ("Vencedor de C1.2") e o
 * que fica no lugar de quem ainda não se sabe. `single`: eliminação simples
 * (uma chave só).
 */
export function useMatchLabels(single: boolean) {
  const t = useTranslations("Bracket");

  function code(key: string) {
    const { part, round, index } = parseMatchKey(key);
    return t(`short.${single ? "single" : part}`, { round, index });
  }

  function roundTitle(
    part: Schemas["BracketPart"],
    round: number,
    lastRound: number,
  ) {
    const name = roundName(part, round, lastRound, single);
    return name.kind === "round"
      ? t("rounds.round", { round: name.round })
      : t(`rounds.${name.kind}`);
  }

  function source(from: Schemas["SideSource"]) {
    return t(`from.${from.outcome}`, { match: code(from.match_key) });
  }

  /** Um lado sem jogador: bye, de onde ele vem ou "a definir". */
  function placeholder(side: Schemas["MatchSide"]) {
    if (side.bye) return t("bye");
    return side.from ? source(side.from) : t("tbd");
  }

  return { code, roundTitle, source, placeholder };
}
