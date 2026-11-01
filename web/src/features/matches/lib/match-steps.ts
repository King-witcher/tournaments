import type { components } from "@/shared/api/schema";

type Detail = components["schemas"]["MatchDetail"];

/** Um passo da série: "Pronto", o veto de mapas, o draft e as partidas. */
export type StepKey = "ready" | "maps" | "champions" | "games";

/**
 * Os passos da série no torneio dela e o atual; `null` sem veto (torneio sem
 * fase nenhuma, que nem pede "Pronto") ou sem nenhum passo feito numa série
 * decidida. Mapas só no modo veto (a votação de cada partida fica no cartão
 * dela, nas Partidas; a da partida 1, quando há draft, abre os Campeões, que
 * dependem do mapa), Campeões só com o draft no site. Série decidida: todos
 * feitos, mas só os que o veto completou (um W.O. no meio dos mapas deixa
 * Pronto e Partidas, sem o pool pela metade).
 */
export function seriesSteps(detail: Detail) {
  const veto = detail.veto;
  if (!veto) return null;
  const all: StepKey[] = ["ready"];
  if (veto.map_mode === "veto") all.push("maps");
  if (veto.champion_phase) all.push("champions");
  all.push("games");
  const at = vetoPhase(detail);
  if (detail.summary.state === "ready") {
    return { steps: all, home: at, current: all.indexOf(at) };
  }
  const reached = vetoReached(veto);
  const steps: StepKey[] = [
    ...all.slice(0, all.indexOf(reached)).filter((s) => s !== "games"),
    "games",
  ];
  if (steps.length < 2) return null;
  return { steps, home: "games" as const, current: steps.length };
}

/**
 * Até onde o veto de uma série decidida chegou, pelo que ele tem: um W.O. no
 * meio deixa o veto em andamento, sem vez.
 */
function vetoReached(veto: NonNullable<Detail["veto"]>): StepKey {
  if (veto.status === "done") return "games";
  if (veto.status === "waiting_ready") return "ready";
  if (veto.map_mode === "veto" && veto.map_steps.length < veto.pool.length - 1)
    return "maps";
  const drafted =
    veto.missing.length === 0 &&
    veto.games.every((g) => (g.pick_a && g.pick_b) || g.team_picks.length > 0);
  return veto.champion_phase && !drafted ? "champions" : "games";
}

/** Onde o veto está: o passo da vez dele, ou as partidas depois dele. */
function vetoPhase(detail: Detail): StepKey {
  const veto = detail.veto;
  if (!veto) return "games";
  if (veto.status === "waiting_ready") return "ready";
  const turn = veto.turn;
  if (veto.status === "done" || !turn) return "games";
  if (turn.target === "champion") return "champions";
  if (veto.map_mode === "veto") return "maps";
  // Votação da partida 1 antes do draft: abre os Campeões.
  const first = veto.games.find((g) => g.game === 1);
  const drafted = first?.pick_a || first?.pick_b || first?.team_picks.length;
  return veto.champion_phase && (veto.game ?? 1) === 1 && !drafted
    ? "champions"
    : "games";
}
