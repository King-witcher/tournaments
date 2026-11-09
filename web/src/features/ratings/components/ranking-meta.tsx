import { getTranslations } from "next-intl/server";

import { Link } from "@/shared/ui/link";
import { RATING_GUIDE_HREF } from "../lib/guide-href";
import { getRanking } from "../server";

/**
 * O metadado do cabeçalho do Ranking: o rating de quem começa, até quantas
 * partidas ele é provisório (o que a etiqueta da tabela quer dizer) e o link
 * do artigo que explica o rating. Fica em `<Suspense>`.
 */
export async function RankingMeta({ slug }: { slug: string }) {
  const [ranking, t] = await Promise.all([
    getRanking(slug),
    getTranslations("Ratings.ranking"),
  ]);
  if (!ranking) return null;
  return t.rich("meta", {
    initial: ranking.initial,
    games: ranking.provisional_games,
    guide: (chunks) => (
      <Link inline href={RATING_GUIDE_HREF}>
        {chunks}
      </Link>
    ),
  });
}
