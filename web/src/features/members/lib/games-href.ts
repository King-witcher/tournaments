/** O filtro da aba Partidas, como fica no endereço. */
export type GamesQuery = {
  tournament?: string;
  map?: string;
  opponent?: number;
  page?: number;
};

/** O endereço da aba Partidas com o filtro (a página 1 e o vazio ficam de fora). */
export function gamesHref(base: string, query: GamesQuery) {
  const params = new URLSearchParams();
  if (query.tournament) params.set("tournament", query.tournament);
  if (query.map) params.set("map", query.map);
  if (query.opponent !== undefined) {
    params.set("opponent", String(query.opponent));
  }
  if (query.page !== undefined && query.page > 1) {
    params.set("page", String(query.page));
  }
  const search = params.toString();
  return `${base}/games${search ? `?${search}` : ""}`;
}

/** Um inteiro positivo do endereço, ou `undefined`. */
function positive(value: string | string[] | undefined) {
  if (typeof value !== "string" || !/^\d+$/.test(value)) return undefined;
  const n = Number(value);
  return Number.isSafeInteger(n) && n > 0 ? n : undefined;
}

/** O filtro lido do endereço; o que não vale fica de fora. */
export function parseGamesQuery(
  search: Record<string, string | string[] | undefined>,
): GamesQuery {
  const text = (value: string | string[] | undefined) =>
    typeof value === "string" && value !== "" ? value : undefined;
  return {
    tournament: text(search.tournament),
    map: text(search.map),
    opponent: positive(search.opponent),
    page: positive(search.page),
  };
}
