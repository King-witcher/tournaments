import type { components } from "@/shared/api/schema";

type Schemas = components["schemas"];
type Veto = Schemas["VetoView"];

/** Um passo da sala: "Pronto", o veto de mapas, o draft e o resultado. */
export type RoomStep = "ready" | "maps" | "champions" | "result";

/** Os passos que a sala tem, em ordem. */
export function roomSteps(veto: Veto): RoomStep[] {
  return [
    "ready",
    ...(veto.map_mode === "veto" ? (["maps"] as const) : []),
    ...(veto.champion_phase ? (["champions"] as const) : []),
    "result",
  ];
}

/** O passo em que a sala está. */
export function roomPhase(veto: Veto): RoomStep {
  if (veto.status === "waiting_ready") return "ready";
  if (veto.status === "done") return "result";
  return veto.turn?.target === "champion" ? "champions" : "maps";
}

/** O passo ainda pede algo de quem vê: o "Pronto" que ele não deu ou a vez dele. */
export function roomAsks(
  veto: Veto,
  mine: Schemas["Side"] | null,
  key: RoomStep,
): boolean {
  if (key === "result" || roomPhase(veto) !== key) return false;
  if (key === "ready") {
    return mine != null && !(mine === "a" ? veto.ready_a : veto.ready_b);
  }
  return veto.options.length > 0 || veto.picks.length > 0;
}
