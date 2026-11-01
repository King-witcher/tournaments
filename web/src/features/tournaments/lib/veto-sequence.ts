import type { components } from "@/shared/api/schema";

/** Um passo do veto de mapas; o mapa que sobra no fim (o decisivo) não é passo. */
export type VetoStep = "ban" | "pick";

/** O que a linha da sequência nomeia: os passos e, no fim, o decisivo. */
export type VetoWord = VetoStep | "decider";

/** Quantos passos iguais seguidos viram "ban ×6" em vez de se repetirem um a um. */
export const GROUPED_FROM = 5;

/**
 * Os passos do veto de mapas de uma melhor de `bestOf` com `pool` mapas, sem o
 * decisivo; o mesmo cálculo do `engine` (`map_sequence`). Sobram `pool - bestOf`
 * bans e `bestOf - 1` picks: com `bans_first`, todos os bans e depois os picks;
 * com `alternating`, dois bans, os picks e o resto dos bans. `null` quando o
 * pool tem menos mapas que partidas.
 */
export function mapSequence(
  pool: number,
  bestOf: number,
  order: components["schemas"]["VetoOrder"],
): VetoStep[] | null {
  if (bestOf < 1 || pool < bestOf) return null;
  const bans = pool - bestOf;
  const first = order === "bans_first" ? bans : Math.min(bans, 2);
  return [
    ...Array<VetoStep>(first).fill("ban"),
    ...Array<VetoStep>(bestOf - 1).fill("pick"),
    ...Array<VetoStep>(bans - first).fill("ban"),
  ];
}

/**
 * A sequência para a linha de texto: o decisivo no fim e, de `GROUPED_FROM`
 * passos iguais seguidos em diante, uma entrada só com a contagem (`count`);
 * os outros saem um a um, com `count` 1.
 */
export function sequenceWords(steps: VetoStep[]) {
  const words: VetoWord[] = [...steps, "decider"];
  const entries: { word: VetoWord; count: number }[] = [];
  for (let i = 0; i < words.length; ) {
    let end = i;
    while (end < words.length && words[end] === words[i]) end++;
    const run = end - i;
    if (run >= GROUPED_FROM) {
      entries.push({ word: words[i], count: run });
    } else {
      for (let n = 0; n < run; n++) entries.push({ word: words[i], count: 1 });
    }
    i = end;
  }
  return entries;
}
