import { getTranslations } from "next-intl/server";

import { getRanking } from "../server";

/**
 * O metadado do cabeçalho do Ranking: o rating de quem começa e até quantas
 * partidas ele é provisório (o que a etiqueta da tabela quer dizer). Fica em
 * `<Suspense>`.
 */
export async function RankingMeta({ slug }: { slug: string }) {
  const [ranking, t] = await Promise.all([
    getRanking(slug),
    getTranslations("Ratings.ranking"),
  ]);
  if (!ranking) return null;
  return t("meta", {
    initial: ranking.initial,
    games: ranking.provisional_games,
  });
}
