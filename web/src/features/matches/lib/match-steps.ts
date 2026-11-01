import type { components } from "@/shared/api/schema";

type Detail = components["schemas"]["MatchDetail"];

/** Um passo da série: "Pronto", o veto de mapas, o draft e as partidas. */
export type StepKey = "ready" | "maps" | "champions" | "games";

/**
 * Os passos da série no torneio dela e o atual; `null` sem veto (torneio sem
 * fase nenhuma, que nem pede "Pronto"). Mapas só no modo veto (a votação de
 * cada partida fica no cartão dela, nas Partidas; a da partida 1, quando há
 * draft, abre os Campeões, que dependem do mapa), Campeões só com o draft no
 * site. Série decidida: todos feitos (`current` além do último).
 */
export function seriesSteps(detail: Detail) {
  const veto = detail.veto;
  if (!veto) return null;
  const steps: StepKey[] = ["ready"];
  if (veto.map_mode === "veto") steps.push("maps");
  if (veto.champion_phase) steps.push("champions");
  steps.push("games");
  const at = currentStep(detail);
  const finished = detail.summary.state !== "ready";
  return {
    steps,
    /** O passo que a tela mostra ao abrir e para onde ela volta. */
    home: at,
    /** Índice do atual no indicador (decidida: além do último). */
    current: finished ? steps.length : steps.indexOf(at),
  };
}

function currentStep(detail: Detail): StepKey {
  const veto = detail.veto;
  if (!veto || detail.summary.state !== "ready") return "games";
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
