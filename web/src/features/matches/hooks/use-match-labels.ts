import { useTranslations } from "next-intl";

import { placementPlace } from "@/features/tournaments/lib/format";
import type { components } from "@/shared/api/schema";
import { parseMatchKey, roundName } from "../lib/bracket-layout";

type Schemas = components["schemas"];

/**
 * Nomes das partidas e rodadas no idioma de quem vê: código curto
 * (`C1.2`; na disputa de posições, o que está em jogo, "Disputa do 3º
 * lugar"), título da rodada, de onde vem um lado ("Vencedor de C1.2") e o
 * que fica no lugar de quem ainda não se sabe. `single`: eliminação simples
 * (uma chave só). `code` diz a fase da 2 em diante; `shortCode` não (para
 * quem já está dentro da chave de uma fase).
 */
export function useMatchLabels(single: boolean) {
  const t = useTranslations("Bracket");

  /** O código sem a fase: dentro da chave de uma fase só, ela já está clara. */
  function shortCode(key: string) {
    const { part, round, index } = parseMatchKey(key);
    return t(`short.${single ? "single" : part}`, {
      round,
      index,
      place: placementPlace(index),
    });
  }

  /** O código, com a fase da 2 em diante ("Fase 2 · Disputa do 3º lugar"). */
  function code(key: string) {
    const { stage } = parseMatchKey(key);
    return stage > 1
      ? t("short.stage", { stage, code: shortCode(key) })
      : shortCode(key);
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
    return t(`from.${from.outcome}`, { match: shortCode(from.match_key) });
  }

  /** Um lado sem jogador: bye, de onde ele vem ou "a definir". */
  function placeholder(side: Schemas["MatchSide"]) {
    if (side.bye) return t("bye");
    return side.from ? source(side.from) : t("tbd");
  }

  return { code, shortCode, roundTitle, source, placeholder };
}
