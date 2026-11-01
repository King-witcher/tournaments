import type { components } from "@/shared/api/schema";

type Schemas = components["schemas"];
type Veto = Schemas["VetoView"];
type Side = Schemas["Side"];

/** Estado de um mapa ou campeão no cartão do veto: a cor nunca vem sozinha (ícone e cinza). */
export type TileState = "available" | "banned" | "picked" | "unavailable";

/** Um mapa do pool no veto de mapas: banido, escolhido para um jogo, o decisivo ou ainda disponível. */
export type PoolMapState =
  | { kind: "banned"; side: Side }
  | { kind: "picked"; side: Side; game: number }
  | { kind: "decider"; game: number }
  | { kind: "available" };

/** O estado de cada mapa do pool (pelo slug) no veto de mapas de uma melhor de `bestOf`. */
export function poolStates(
  veto: Veto,
  bestOf: number,
): Record<string, PoolMapState> {
  const states: Record<string, PoolMapState> = {};
  let picks = 0;
  for (const step of veto.map_steps) {
    if (step.action === "ban") {
      states[step.map.slug] = { kind: "banned", side: step.side };
    } else {
      picks++;
      states[step.map.slug] = { kind: "picked", side: step.side, game: picks };
    }
  }
  const done = veto.map_steps.length === veto.pool.length - 1;
  for (const m of veto.pool) {
    if (states[m.slug]) continue;
    states[m.slug] =
      done && veto.remaining.some((r) => r.slug === m.slug)
        ? { kind: "decider", game: bestOf }
        : { kind: "available" };
  }
  return states;
}

/** A chave de texto do passo esperado (`ban_map`, `pick_champion`, `vote_map`...). */
export function turnKey(turn: Schemas["VetoTurn"]) {
  return `${turn.action}_${turn.target}` as const;
}

/** O jogo em andamento no veto (1 antes de começar). */
export const currentGame = (veto: Veto) => veto.game ?? 1;

/** O lado adversário. */
export const otherSide = (side: Side): Side => (side === "a" ? "b" : "a");

/**
 * `a` é mais nova que `b`: a API montou `a` depois (`now`, a hora dela em
 * cada resposta; uma instância só). Uma recarga que saiu antes do clique
 * perde para a resposta do clique, e a do veto apagado e refeito ganha.
 */
export function isNewer(
  a: Schemas["MatchDetail"],
  b: Schemas["MatchDetail"],
): boolean {
  return Date.parse(a.now) > Date.parse(b.now);
}
