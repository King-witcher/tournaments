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
