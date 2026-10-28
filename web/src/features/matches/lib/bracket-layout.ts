import type { components } from "@/shared/api/schema";

type Schemas = components["schemas"];
type Match = Schemas["BracketMatch"];
type Part = Schemas["BracketPart"];

/** Uma coluna da chave: as partidas de uma rodada de uma parte. */
export type Column = {
  part: Part;
  round: number;
  /** A última rodada da parte, para o nome da rodada (final, semifinal…). */
  lastRound: number;
  matches: Match[];
};

/** Uma linha entre partidas: de onde vem quem ocupa um lado de `to`. */
export type Edge = { from: string; to: string; side: "a" | "b" };

/**
 * As colunas das partes pedidas, na ordem delas e das rodadas, lado a lado
 * (a chave principal e, à direita, a grande final).
 */
export function bracketColumns(matches: Match[], parts: Part[]): Column[] {
  return parts.flatMap((part) => {
    const rounds = new Map<number, Match[]>();
    for (const m of matches.filter((m) => m.part === part)) {
      rounds.set(m.round, [...(rounds.get(m.round) ?? []), m]);
    }
    const lastRound = Math.max(0, ...rounds.keys());
    return [...rounds.entries()]
      .sort(([a], [b]) => a - b)
      .map(([round, list]) => ({
        part,
        round,
        lastRound,
        matches: list.sort((a, b) => a.index - b.index),
      }));
  });
}

/**
 * As linhas entre as partidas de um grupo de colunas: cada lado que vem de
 * outra partida do mesmo grupo. Quem cai da chave principal para a
 * repescagem (ou vem dela para a grande final) muda de aba e fica só com o
 * texto ("Perdedor de C2.1"). O reset que não precisou não recebe linha.
 */
export function bracketEdges(columns: Column[]): Edge[] {
  const matches = columns.flatMap((c) => c.matches);
  const keys = new Set(matches.map((m) => m.key));
  return matches.flatMap((m) =>
    m.state === "not_needed"
      ? []
      : (["a", "b"] as const).flatMap((side) => {
          const from = m[side].from?.match_key;
          return from && keys.has(from) ? [{ from, to: m.key, side }] : [];
        }),
  );
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
