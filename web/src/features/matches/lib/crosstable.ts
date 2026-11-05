import type { components } from "@/shared/api/schema";
import { matchProgress } from "./status";

type Match = components["schemas"]["BracketMatch"];

/** Um time na tabela cruzada: é ao mesmo tempo uma linha e uma coluna. */
export type CrossTeam = {
  id: number;
  position: number;
  name: string;
  avatar?: string | null;
};

/** O que uma célula da tabela cruzada mostra, do ponto de vista da linha. */
export type CrossResult =
  | { kind: "pending" }
  | { kind: "playing"; own: number; other: number }
  | { kind: "walkover"; won: boolean }
  | {
      kind: "score";
      /** `drawn`: a série par empatada, sem vencedor. */
      outcome: "won" | "lost" | "drawn";
      own: number;
      other: number;
      frags: boolean;
    };

function pairKey(x: number, y: number) {
  return x < y ? `${x}-${y}` : `${y}-${x}`;
}

/**
 * As séries pelos dois times, em qualquer ordem: o round robin tem uma por par
 * (a folga, sem o outro lado, fica de fora).
 */
export function seriesByPair(matches: Match[]) {
  const pairs = new Map<string, Match>();
  for (const m of matches) {
    if (m.a.team_id != null && m.b.team_id != null) {
      pairs.set(pairKey(m.a.team_id, m.b.team_id), m);
    }
  }
  return (x: number, y: number) => pairs.get(pairKey(x, y));
}

/**
 * A série lida pelo time `team`: o lado `a` ou `b` é do calendário, não da
 * tabela. Na md1, os frags, porque 1–0 diz pouco (sem frags, o placar da
 * série); na maior, o placar da série. Em andamento, os mapas confirmados.
 */
export function crossResult(m: Match, team: number): CrossResult {
  const isA = m.a.team_id === team;
  const r = m.result;
  if (r) {
    // O lado que venceu é do calendário; sem vencedor, a série empatou.
    const won = r.winner != null && (r.winner === "a") === isA;
    if (r.walkover) return { kind: "walkover", won };
    const frags = m.best_of === 1 && m.a_frags != null && m.b_frags != null;
    const a = (frags ? m.a_frags : r.a_wins) ?? 0;
    const b = (frags ? m.b_frags : r.b_wins) ?? 0;
    return {
      kind: "score",
      outcome: r.winner == null ? "drawn" : won ? "won" : "lost",
      own: isA ? a : b,
      other: isA ? b : a,
      frags,
    };
  }
  if (matchProgress(m) === "pending") return { kind: "pending" };
  return {
    kind: "playing",
    own: isA ? m.a_maps : m.b_maps,
    other: isA ? m.b_maps : m.a_maps,
  };
}
