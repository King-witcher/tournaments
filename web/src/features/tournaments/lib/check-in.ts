import type { components } from "@/shared/api/schema";

/** Check-in aberto e quem vê, inscrito, ainda não fez: fica fora da chave se não fizer. */
export function checkInPending(
  t: Pick<
    components["schemas"]["TournamentSummary"],
    "status" | "registered" | "checked_in"
  >,
) {
  return t.status === "check_in" && t.registered && !t.checked_in;
}
