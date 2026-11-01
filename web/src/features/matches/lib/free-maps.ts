import type { components } from "@/shared/api/schema";

type Schemas = components["schemas"];

/**
 * Os mapas que ainda cabem na partida `number`: fora os das outras partidas
 * da série, que não se repetem (a API recusa de novo). O da própria partida
 * fica, para quem corrige. Passe o número do próximo mapa para tirar todos.
 */
export function freeMaps(
  maps: Schemas["Map"][],
  games: Schemas["GameDetail"][],
  number: number,
) {
  const played = new Set(
    games.filter((g) => g.number !== number).map((g) => g.map?.slug),
  );
  return maps.filter((m) => !played.has(m.slug));
}
