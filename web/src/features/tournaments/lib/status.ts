import type { components } from "@/shared/api/schema";

type Status = components["schemas"]["TournamentStatus"];

/** Formato, vagas e campeões ainda mudam (a API confere de novo). */
export function setupEditable(status: Status) {
  return (
    status === "draft" || status === "registration" || status === "check_in"
  );
}

/** Textos, traduções e visibilidade ainda mudam. */
export function textsEditable(status: Status) {
  return status !== "finished";
}

/** O organizador ainda inscreve e tira participantes (até começar). */
export function participantsEditable(status: Status) {
  return setupEditable(status);
}

/** Dá para gerar link de inscrição: no rascunho e com a inscrição aberta. */
export function invitesAllowed(status: Status) {
  return status === "draft" || status === "registration";
}

/** Já começou: a chave, as partidas e a classificação existem. */
export function hasStarted(status: Status) {
  return status === "in_progress" || status === "finished";
}

/**
 * A ordem do seeding se revisa agora: no check-in ou, em torneio sem
 * check-in, com a inscrição aberta (antes disso, não se sabe quem entra).
 */
export function seedingOpen(
  tournament: Pick<
    components["schemas"]["Tournament"],
    "status" | "check_in_minutes"
  >,
) {
  return (
    tournament.status === "check_in" ||
    (tournament.status === "registration" &&
      tournament.check_in_minutes == null)
  );
}
