/** A partida em que uma ação age. */
export type MatchTarget = { tenant: string; slug: string; matchKey: string };

/** Parâmetros de caminho das rotas `.../matches/{match}/...`. */
export function matchParams({ tenant, slug, matchKey }: MatchTarget) {
  return { path: { slug: tenant, tournament: slug, match: matchKey } };
}

/** Uma linha do detalhe por mapa no formulário (frags como digitados). */
export type GameValues = { map: string; a: string; b: string };

/** A linha como a API recebe. */
export function gameInput(g: GameValues) {
  return { map: g.map, a_frags: Number(g.a) || 0, b_frags: Number(g.b) || 0 };
}
