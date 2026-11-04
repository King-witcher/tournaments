import type { components } from "@/shared/api/schema";
import type { TileState } from "./veto";

type Schemas = components["schemas"];

/**
 * Um campeão na grade do draft: só responde "posso escolher agora?". Banido,
 * fora das opções de quem vê (`unavailable`, com o motivo) ou disponível. Quem
 * escolheu o quê fica no resumo do draft.
 */
export type ChampionTile = {
  state: Extract<TileState, "available" | "banned" | "unavailable">;
  /** Banido: quem baniu. */
  side?: Schemas["Side"];
  /** Fora do pick de quem vê, sem ser banido. */
  reason?: Exclude<Schemas["VetoUnavailableReason"], "banned">;
};

/**
 * O estado de cada campeão (pelo slug) na grade do draft: banido; fora do
 * pick de quem vê pela regra (`slot`, só no pick, e só para quem age:
 * espectador e gestão não o recebem); disponível. O escolhido por outro que
 * ainda dá para escolher (espelho) continua disponível.
 */
export function championTiles(
  veto: Schemas["VetoView"],
  slot: Schemas["VetoPickSlot"] | undefined,
): Record<string, ChampionTile> {
  const reasons = new Map(
    (slot?.unavailable ?? []).map((u) => [u.slug, u.reason]),
  );
  const tiles: Record<string, ChampionTile> = {};
  for (const c of veto.champions) {
    const reason = reasons.get(c.slug);
    tiles[c.slug] = c.banned_by
      ? { state: "banned", side: c.banned_by }
      : reason == null || reason === "banned"
        ? { state: "available" }
        : { state: "unavailable", reason };
  }
  return tiles;
}
