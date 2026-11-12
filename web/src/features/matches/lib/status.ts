import type { components } from "@/shared/api/schema";
import type { Status } from "@/shared/ui/status-tag";

type Schemas = components["schemas"];

/** O que pede uma ação de quem vê (enviar, responder, resolver, o veto). */
const ACTIONABLE = [
  "report",
  "respond",
  "resolve",
  "accept_contest",
  "ready",
  "veto",
] as const satisfies Schemas["ViewerTask"][];

export type ActionableTask = (typeof ACTIONABLE)[number];

export function isActionable(
  task: Schemas["ViewerTask"] | null | undefined,
): task is ActionableTask {
  return ACTIONABLE.some((actionable) => actionable === task);
}

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
    (m): m is typeof m & { task: ActionableTask } =>
      m.key !== exclude && isActionable(m.task) && !m.opponent_playing,
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

/** A contestação em aberto na série de quem vê: resolver, aceitar ou esperar a organização. */
const CONTEST = [
  "resolve",
  "accept_contest",
  "await_organizer",
] as const satisfies Schemas["ViewerTask"][];

/**
 * A etiqueta do que a série espera de quem vê, só onde ela chama atenção: a
 * contestação, na cor do contestado, e a ação que falta a quem joga, em
 * roxo. Esperar o outro lado é o caminho normal e não tem etiqueta
 * (`waitingKey` dá o texto).
 */
export function taskTag(task: Schemas["ViewerTask"]): Status | null {
  if (CONTEST.some((contest) => contest === task)) return "contested";
  return isActionable(task) ? "action" : null;
}

/**
 * A chave do texto (em `Bracket`) do que a série espera de outro: o adversário
 * em outra série, o adversário, a organização, o capitão, o time. A ação de
 * quem vê não entra (`undefined`): ela é um aviso com o botão.
 */
export function waitingKey(m: {
  task?: Schemas["ViewerTask"] | null;
  veto?: Schemas["VetoStatus"] | null;
  opponent_playing?: boolean;
}) {
  if (m.opponent_playing) return "opponentPlaying";
  if (!m.task || isActionable(m.task)) return undefined;
  return `task.${taskKey(m.task, m.veto)}` as const;
}
