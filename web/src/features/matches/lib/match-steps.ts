import type { components } from "@/shared/api/schema";

type Detail = components["schemas"]["MatchDetail"];
type Veto = NonNullable<Detail["veto"]>;

/** Um passo da série: "Pronto", o veto de mapas, o draft e cada partida. */
export type StepKey = "ready" | "maps" | "champions" | `game-${number}`;

/** O passo da partida `number`. */
export const gameStep = (number: number): StepKey => `game-${number}`;

/** A partida de um passo, ou `null` nos passos do veto. */
export function stepGame(key: StepKey): number | null {
  return key.startsWith("game-") ? Number(key.slice(5)) : null;
}

/** As tarefas de uma partida enviada que pedem uma ação de quem vê. */
const GAME_ACTIONS: components["schemas"]["ViewerTask"][] = [
  "respond",
  "resolve",
  "accept_contest",
];

/** Os passos da série, o atual e o que abre (`seriesSteps`). */
export type SeriesFlow = {
  steps: StepKey[];
  /** Índice do passo em que a série está (`steps.length` na decidida). */
  current: number;
  /** O passo que abre; `null` sem passos. */
  home: StepKey | null;
  /** O passo do veto em andamento, mesmo antes do atual (o substituto do TDM escolhe o campeão depois da partida 1). */
  active: "ready" | "maps" | "champions" | null;
};

/**
 * Os passos da série e onde ela está: Pronto, Mapas (só no modo veto) e
 * Campeões (só com o draft no site), quando o torneio tem veto, e uma
 * partida por passo, até o melhor de N. `home` é o passo em que quem vê tem
 * o que fazer (a vez dele no veto, que tem prazo, antes de confirmar ou
 * resolver uma partida enviada), senão o atual. Com partida enviada, a
 * série fica nas partidas mesmo que o veto volte aos campeões (o
 * substituto do TDM escolhendo na partida 2): o passo Campeões abre para
 * ele, e as partidas enviadas continuam alcançáveis. Série decidida: só os
 * passos do veto que ele completou (um W.O. no meio dos mapas deixa o
 * Pronto) e as partidas jogadas, abrindo na última. Sem lados ainda (ou sem
 * adversário), nenhum passo.
 */
export function seriesSteps(detail: Detail): SeriesFlow {
  const veto = detail.veto;
  const s = detail.summary;
  const vetoSteps: StepKey[] = [];
  if (veto) {
    vetoSteps.push("ready");
    if (veto.map_mode === "veto") vetoSteps.push("maps");
    if (veto.champion_phase) vetoSteps.push("champions");
  }

  if (s.state === "completed") {
    const reached = veto ? vetoReached(veto) : "games";
    const done =
      reached === "games"
        ? vetoSteps
        : vetoSteps.slice(0, vetoSteps.indexOf(reached));
    const steps = [...done, ...detail.games.map((g) => gameStep(g.number))];
    return {
      steps,
      current: steps.length,
      home: steps[steps.length - 1] ?? null,
      active: null,
    };
  }
  if (s.state !== "ready") {
    return { steps: [], current: 0, home: null, active: null };
  }

  const games = Array.from({ length: s.best_of }, (_, i) => gameStep(i + 1));
  const steps = [...vetoSteps, ...games];
  const phase = vetoPhase(detail);
  const active = phase === "games" ? null : phase;
  const playing = gameStep(
    detail.next_game ?? Math.max(1, detail.games.length),
  );
  const at = active && detail.games.length === 0 ? active : playing;
  const respond = detail.games.find(
    (g) => g.task != null && GAME_ACTIONS.includes(g.task),
  );
  const mine = vetoTurnIsMine(detail);
  const home =
    mine && active ? active : respond && !mine ? gameStep(respond.number) : at;
  const index = steps.indexOf(at);
  return {
    steps,
    current: index < 0 ? steps.length - 1 : index,
    home,
    active,
  };
}

/**
 * O passo ainda pede algo de quem vê: o "Pronto" que ele não deu, a vez dele
 * no veto (no passo em que o veto está), a partida da vez para quem pode enviar,
 * a enviada que ele confirma ou resolve.
 */
export function asks(detail: Detail, key: StepKey | null): boolean {
  const veto = detail.veto;
  const number = key ? stepGame(key) : null;
  if (key == null) return false;
  if (number == null) {
    if (!veto || veto.status === "done") return false;
    if (key === "ready")
      return veto.status === "waiting_ready" && veto.can_act.ready;
    return vetoPhase(detail) === key && vetoTurnIsMine(detail);
  }
  const game = detail.games.find((g) => g.number === number);
  if (game) return game.task != null && GAME_ACTIONS.includes(game.task);
  return (
    detail.can_report &&
    detail.summary.state === "ready" &&
    detail.next_game === number
  );
}

/** Quem vê tem um passo do veto para fazer agora (com prazo). */
function vetoTurnIsMine(detail: Detail) {
  const veto = detail.veto;
  return (
    veto?.status === "in_progress" &&
    (veto.options.length > 0 || veto.picks.length > 0)
  );
}

/**
 * Até onde o veto de uma série decidida chegou, pelo que ele tem: um W.O. no
 * meio deixa o veto em andamento, sem vez.
 */
function vetoReached(veto: Veto): "ready" | "maps" | "champions" | "games" {
  if (veto.status === "done") return "games";
  if (veto.status === "waiting_ready") return "ready";
  if (veto.map_mode === "veto" && veto.map_steps.length < veto.pool.length - 1)
    return "maps";
  const drafted =
    veto.missing.length === 0 &&
    veto.games.every(
      (g) =>
        (g.pick_a && g.pick_b) ||
        g.team_picks.length > 0 ||
        g.hidden_picks.length > 0,
    );
  return veto.champion_phase && !drafted ? "champions" : "games";
}

/**
 * Onde o veto está: o passo da vez dele, ou as partidas (a votação de cada
 * partida fica no passo dela; a da partida 1, com o draft, abre os Campeões,
 * que dependem do mapa).
 */
export function vetoPhase(
  detail: Detail,
): "ready" | "maps" | "champions" | "games" {
  const veto = detail.veto;
  if (!veto) return "games";
  if (veto.status === "waiting_ready") return "ready";
  const turn = veto.turn;
  if (veto.status === "done" || !turn) return "games";
  if (turn.target === "champion") return "champions";
  if (veto.map_mode === "veto") return "maps";
  const first = veto.games.find((g) => g.game === 1);
  const drafted =
    first?.pick_a ||
    first?.pick_b ||
    first?.team_picks.length ||
    first?.hidden_picks.length;
  return veto.champion_phase && (veto.game ?? 1) === 1 && !drafted
    ? "champions"
    : "games";
}
