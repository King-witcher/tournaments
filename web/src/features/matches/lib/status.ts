import type { components } from "@/shared/api/schema";
import type { Status } from "@/shared/ui/status-tag";

type Schemas = components["schemas"];

/** O que pede uma ação de quem vê (enviar, responder, resolver, o veto). */
export const ACTIONABLE: Schemas["ViewerTask"][] = [
  "report",
  "respond",
  "resolve",
  "ready",
  "veto",
];

/** Onde a série está, para achar de relance o que falta jogar. */
export type MatchProgress = "completed" | "in_progress" | "pending";

/**
 * Concluída (também no W.O.), em andamento (veto começado, mapa enviado ou
 * confirmado, série em jogo) ou pendente. Bye e reset que não precisou não se
 * jogam: quem mostra não pergunta por eles.
 */
export function matchProgress(m: Schemas["BracketMatch"]): MatchProgress {
  if (m.state === "completed") return "completed";
  const begun =
    m.in_progress ||
    m.veto === "in_progress" ||
    m.veto === "done" ||
    m.a_maps + m.b_maps + m.pending_maps + m.contested_maps > 0;
  return begun ? "in_progress" : "pending";
}

/**
 * As séries que esperam uma ação de quem vê, menos a `exclude`. Com o
 * adversário ocupado em outra (round robin), a série ainda não espera ninguém.
 */
export function actionableMatches(
  bracket: Schemas["TournamentBracket"],
  exclude?: string,
) {
  return bracket.matches.filter(
    (m) =>
      m.key !== exclude &&
      m.task != null &&
      ACTIONABLE.includes(m.task) &&
      !m.opponent_playing,
  );
}

/**
 * A chave do texto (em `Bracket.task`) do que a partida espera de quem vê:
 * esperar o adversário vira, com veto, esperar ele ficar pronto ou o veto em
 * andamento.
 */
export function taskKey(
  task: Schemas["ViewerTask"],
  veto: Schemas["VetoStatus"] | null | undefined,
) {
  if (task === "await_opponent" && veto === "waiting_ready")
    return "await_ready";
  if (task === "await_opponent" && veto === "in_progress")
    return "veto_in_progress";
  return task;
}

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
  await_captain: "waiting",
  await_team: "waiting",
  ready: "check_in",
  veto: "check_in",
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
