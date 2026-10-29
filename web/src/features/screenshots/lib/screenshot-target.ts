/** O mapa em que um screenshot entra: a partida e o número do mapa na série. */
export type ScreenshotTarget = {
  tenant: string;
  slug: string;
  matchKey: string;
  number: number;
};

/** Parâmetros de caminho das rotas `.../matches/{match}/games/{game}/screenshots...`. */
export function screenshotParams({
  tenant,
  slug,
  matchKey,
  number,
}: ScreenshotTarget) {
  return {
    path: { slug: tenant, tournament: slug, match: matchKey, game: number },
  };
}
