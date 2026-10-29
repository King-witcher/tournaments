import type { components } from "@/shared/api/schema";

type Schemas = components["schemas"];
type Match = Schemas["BracketMatch"];

/** Ordem das partes na chave: quem cai da de cima ainda joga a de baixo. */
const PART_ORDER: Record<Schemas["BracketPart"], number> = {
  winners: 0,
  losers: 1,
  grand_final: 2,
  // Sem eliminação: a tabela classifica.
  round_robin: 0,
  swiss: 0,
};

const later = (a: Match, b: Match) =>
  PART_ORDER[a.part] - PART_ORDER[b.part] || a.round - b.round;

/**
 * A série que eliminou o time `teamId`: a última que ele perdeu, na ordem da
 * chave (na dupla, a derrota na chave de cima não elimina). Sem derrota (o
 * campeão), nada.
 */
export function eliminationOf(matches: Match[], teamId: number) {
  let found: { match: Match; side: Schemas["Side"] } | undefined;
  for (const m of matches) {
    if (m.state !== "completed" || !m.result) continue;
    const side =
      m.a.team_id === teamId ? "a" : m.b.team_id === teamId ? "b" : null;
    if (side === null || m.result.winner === side) continue;
    if (!found || later(m, found.match) > 0) found = { match: m, side };
  }
  return found;
}
