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

/** Um mapa no formulário: slug do mapa e frags de cada lado (vazio é `null`). */
export type GameValues = {
  map: string;
  a: number | null;
  b: number | null;
};

export const EMPTY_GAME: GameValues = {
  map: "",
  a: null,
  b: null,
};

/** O mapa como a API recebe (o vencedor sai do placar). Sem mapa vai nulo: `match.map_required`. */
export function gameInput(g: { map: string; a: number; b: number }) {
  return { map: g.map || null, a_frags: g.a, b_frags: g.b };
}

/** Id da linha de um mapa na página da partida: o foco vai para ela depois de agir no mapa. */
export const gameRowId = (number: number) => `game-${number}`;

/** Id do placar da série: o foco vai para ele depois do W.O., que fecha o bloco da organização. */
export const SCORE_ID = "match-score";

/** Id do bloco do veto: o foco vai para ele depois de um passo (o botão some). */
export const VETO_ID = "match-veto";
