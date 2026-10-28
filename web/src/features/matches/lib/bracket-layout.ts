import type { components } from "@/shared/api/schema";

type Schemas = components["schemas"];
type Match = Schemas["BracketMatch"];
type Part = Schemas["BracketPart"];

export type Round = { round: number; matches: Match[] };
export type PartRounds = { part: Part; rounds: Round[] };

const PARTS: Part[] = ["winners", "losers", "grand_final"];

/** Partidas por parte da chave (cima, baixo, grande final) e rodada. */
export function groupByRound(matches: Match[]): PartRounds[] {
  return PARTS.map((part) => {
    const rounds = new Map<number, Match[]>();
    for (const m of matches.filter((m) => m.part === part)) {
      rounds.set(m.round, [...(rounds.get(m.round) ?? []), m]);
    }
    return {
      part,
      rounds: [...rounds.entries()]
        .sort(([a], [b]) => a - b)
        .map(([round, list]) => ({
          round,
          matches: list.sort((a, b) => a.index - b.index),
        })),
    };
  }).filter((p) => p.rounds.length > 0);
}

/**
 * Nome da rodada: na simples e no fim de cada chave, final, semifinal e
 * quartas; na grande final, a série e o reset; nas outras, o número.
 */
export type RoundName =
  | { kind: "final" | "semifinal" | "quarterfinal" | "grandFinal" | "reset" }
  | { kind: "round"; round: number };

export function roundName(
  part: Part,
  round: number,
  lastRound: number,
  single: boolean,
): RoundName {
  if (part === "grand_final") {
    return { kind: round === 1 ? "grandFinal" : "reset" };
  }
  const fromEnd = lastRound - round;
  if (fromEnd === 0) return { kind: "final" };
  if (single && fromEnd === 1) return { kind: "semifinal" };
  if (single && fromEnd === 2) return { kind: "quarterfinal" };
  return { kind: "round", round };
}

/** `wb-1-3` → parte, rodada e índice (o formato que a API gera). */
export function parseMatchKey(key: string) {
  const [prefix, round, index] = key.split("-");
  const part: Part =
    prefix === "lb" ? "losers" : prefix === "gf" ? "grand_final" : "winners";
  return { part, round: Number(round), index: Number(index) };
}
