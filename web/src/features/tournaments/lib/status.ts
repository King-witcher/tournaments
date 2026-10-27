import type { components } from "@/shared/api/schema";

type Status = components["schemas"]["TournamentStatus"];

/** Formato, vagas e campeões ainda mudam (a API confere de novo). */
export function setupEditable(status: Status) {
  return (
    status === "draft" || status === "registration" || status === "check_in"
  );
}

/** Textos e traduções ainda mudam. */
export function textsEditable(status: Status) {
  return status !== "finished";
}
