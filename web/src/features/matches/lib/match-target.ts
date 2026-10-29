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

/** Um mapa no formulário: vencedor, slug do mapa e frags (vazio é `null`). */
export type GameValues = {
  winner: "a" | "b" | "";
  map: string;
  a: number | null;
  b: number | null;
};

export const EMPTY_GAME: GameValues = {
  winner: "",
  map: "",
  a: null,
  b: null,
};

/** O mapa como a API recebe (frags de um lado só voltam como erro da API). */
export function gameInput(g: GameValues & { winner: "a" | "b" }) {
  return {
    winner: g.winner,
    map: g.map || null,
    a_frags: g.a,
    b_frags: g.b,
  };
}

/** Id da linha de um mapa na página da partida: o foco vai para ela depois de agir no mapa. */
export const gameRowId = (number: number) => `game-${number}`;

/** Id do placar da série: o foco vai para ele depois do W.O., que fecha o bloco da organização. */
export const SCORE_ID = "match-score";

/** Id do bloco do veto: o foco vai para ele depois de um passo (o botão some). */
export const VETO_ID = "match-veto";
