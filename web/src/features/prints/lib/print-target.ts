/** O mapa em que um print entra: a partida e o número do mapa na série. */
export type PrintTarget = {
  tenant: string;
  slug: string;
  matchKey: string;
  number: number;
};

/** Parâmetros de caminho das rotas `.../matches/{match}/games/{game}/prints...`. */
export function printParams({ tenant, slug, matchKey, number }: PrintTarget) {
  return {
    path: { slug: tenant, tournament: slug, match: matchKey, game: number },
  };
}
