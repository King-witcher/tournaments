import type { components } from "@/shared/api/schema";

type Status = components["schemas"]["TournamentStatus"];

/** Formato, vagas e campeões ainda mudam (a API confere de novo). */
export function setupEditable(status: Status) {
  return (
    status === "draft" || status === "registration" || status === "check_in"
  );
}

/** Textos, traduções e o tipo de inscrição ainda mudam. */
export function textsEditable(status: Status) {
  return status !== "finished";
}

/** O organizador ainda inscreve e tira participantes (até começar). */
export function participantsEditable(status: Status) {
  return setupEditable(status);
}

/** Dá para gerar link de inscrição: no rascunho e até o início. */
export function invitesAllowed(status: Status) {
  return (
    status === "draft" || status === "registration" || status === "check_in"
  );
}

/** A inscrição está aberta: até o início, também durante o check-in. */
export function registrationOpen(status: Status) {
  return status === "registration" || status === "check_in";
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
