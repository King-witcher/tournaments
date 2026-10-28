import type { components } from "@/shared/api/schema";
import type { Status } from "@/shared/ui/status-tag";

type Schemas = components["schemas"];

/** O que pede uma ação de quem vê (enviar, responder, resolver). */
export const ACTIONABLE: Schemas["ViewerTask"][] = [
  "report",
  "respond",
  "resolve",
];

/**
 * Cor da etiqueta do que a partida espera de quem vê. A ação que falta a
 * quem joga tem o destaque do check-in que falta (roxo); a contestação para
 * a organização resolver, o do contestado; esperar o outro lado, o de
 * aguardando.
 */
export const TASK_STATUS: Record<Schemas["ViewerTask"], Status> = {
  report: "check_in",
  respond: "check_in",
  resolve: "contested",
  await_opponent: "waiting",
  await_organizer: "waiting",
};

/** Cor da etiqueta de um mapa da série. */
export const GAME_STATUS: Record<Schemas["GameStatus"], Status> = {
  pending: "waiting",
  contested: "contested",
  confirmed: "confirmed",
};

/** Cor da etiqueta do estado da partida. */
export const MATCH_STATE_STATUS: Record<Schemas["MatchState"], Status> = {
  pending: "neutral",
  ready: "in_progress",
  bye: "neutral",
  completed: "finished",
  not_needed: "neutral",
};
