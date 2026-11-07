import type { components } from "@/shared/api/schema";

type Schemas = components["schemas"];
type Candidate = Schemas["StageCandidate"];
type Entry = Pick<Schemas["StageEntry"], "status" | "check_in" | "candidates">;

/**
 * Onde o time está na fase seguinte, antes de ela começar: `enters` (tem
 * ordem), `withdrawn` (fora direto), `absent` (o check-in está aberto e ele
 * não confirmou) ou `substitute` (abaixo do corte: sobe se alguém de cima
 * faltar).
 */
export type CandidateState = "enters" | "substitute" | "absent" | "withdrawn";

export function candidateState(
  candidate: Candidate,
  entry: Pick<Entry, "status" | "check_in">,
): CandidateState {
  if (candidate.withdrawn) return "withdrawn";
  if (candidate.projected_seed != null) return "enters";
  if (entry.check_in && entry.status === "check_in" && !candidate.checked_in) {
    return "absent";
  }
  return "substitute";
}

/** Quem entraria agora, na ordem da fase. */
export function entering(entry: Pick<Entry, "candidates">) {
  return entry.candidates
    .filter((candidate) => candidate.projected_seed != null)
    .toSorted((a, b) => (a.projected_seed ?? 0) - (b.projected_seed ?? 0));
}

/** Os suplentes, na ordem da tabela da fase anterior. */
export function substitutes(entry: Entry) {
  return entry.candidates.filter(
    (candidate) => candidateState(candidate, entry) === "substitute",
  );
}
