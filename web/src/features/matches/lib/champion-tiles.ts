import type { components } from "@/shared/api/schema";
import type { TileState } from "./veto";

type Schemas = components["schemas"];

/** Um campeão na grade do draft: o estado, as partidas em que já foi escolhido e por quem, e por que não dá. */
export type ChampionTile = {
  state: TileState;
  /** Banido: quem baniu; escolhido: quem escolheu primeiro. */
  side?: Schemas["Side"];
  /** As partidas da série em que ele já foi escolhido. */
  games: number[];
  /** Fora das opções do pick de quem vê, sem ser banido nem escolhido na série. */
  reason?: Exclude<Schemas["VetoUnavailableReason"], "banned">;
};

/**
 * O estado de cada campeão (pelo slug) na grade do draft: banido; escolhido
 * para alguma partida da série (mesmo que ainda dê para escolher, com
 * espelho); fora do pick de quem vê pela regra (`slot`, só no pick);
 * disponível.
 */
export function championTiles(
  veto: Schemas["VetoView"],
  slot: Schemas["VetoPickSlot"] | undefined,
): Record<string, ChampionTile> {
  const picked = new Map<string, { side: Schemas["Side"]; games: number[] }>();
  const add = (slug: string, side: Schemas["Side"], game: number) => {
    const seen = picked.get(slug) ?? { side, games: [] };
    if (!seen.games.includes(game)) seen.games.push(game);
    picked.set(slug, seen);
  };
  for (const g of veto.games) {
    if (g.pick_a) add(g.pick_a.slug, "a", g.game);
    if (g.pick_b) add(g.pick_b.slug, "b", g.game);
    for (const p of g.team_picks) add(p.champion.slug, p.side, g.game);
  }
  const reasons = new Map(
    (slot?.unavailable ?? []).map((u) => [u.slug, u.reason]),
  );
  const tiles: Record<string, ChampionTile> = {};
  for (const c of veto.champions) {
    const seen = picked.get(c.slug);
    const reason = reasons.get(c.slug);
    tiles[c.slug] = c.banned_by
      ? { state: "banned", side: c.banned_by, games: [] }
      : seen
        ? { state: "picked", side: seen.side, games: seen.games }
        : reason == null || reason === "banned"
          ? { state: "available", games: [] }
          : { state: "unavailable", games: [], reason };
  }
  return tiles;
}
