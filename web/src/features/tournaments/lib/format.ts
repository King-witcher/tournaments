import type { components } from "@/shared/api/schema";

type Schemas = components["schemas"];

/**
 * A chave como os textos a nomeiam e explicam: a dupla com reset na grande
 * final tem nome e explicação próprios.
 */
export function bracketKind(
  stage: Pick<Schemas["StageFormat"], "bracket" | "grand_final_reset">,
) {
  return stage.bracket === "double" && stage.grand_final_reset
    ? "doubleReset"
    : stage.bracket;
}

/** Até onde o campeão não se repete; `off` sem a regra. */
export function repeatScope(rules: Schemas["ChampionRules"]) {
  return rules.no_repeat?.scope ?? "off";
}

/** Mapas que vencem uma série melhor de N (N ímpar): 2 na md3, 3 na md5. */
export function mapsToWin(bestOf: number) {
  return Math.floor(bestOf / 2) + 1;
}
