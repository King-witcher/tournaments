/** A partida em que uma ação age. */
export type MatchTarget = { tenant: string; slug: string; matchKey: string };

/** Parâmetros de caminho das rotas `.../matches/{match}/...`. */
export function matchParams({ tenant, slug, matchKey }: MatchTarget) {
  return { path: { slug: tenant, tournament: slug, match: matchKey } };
}

/** Parâmetros de caminho das rotas `.../matches/{match}/games/{game}...`. */
export function gameParams(target: MatchTarget, game: number) {
  return { path: { ...matchParams(target).path, game } };
}

/** Um mapa no formulário: vencedor, slug do mapa e frags como digitados. */
export type GameValues = {
  winner: "a" | "b" | "";
  map: string;
  a: string;
  b: string;
};

export const EMPTY_GAME: GameValues = { winner: "", map: "", a: "", b: "" };

/** O mapa como a API recebe (frags de um lado só voltam como erro da API). */
export function gameInput(g: GameValues & { winner: "a" | "b" }) {
  return {
    winner: g.winner,
    map: g.map || null,
    a_frags: g.a.trim() ? Number(g.a) : null,
    b_frags: g.b.trim() ? Number(g.b) : null,
  };
}

/** Id da linha de um mapa na página da partida: o foco vai para ela depois de agir no mapa. */
export const gameRowId = (number: number) => `game-${number}`;
