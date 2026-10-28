import { getTranslations } from "next-intl/server";

import { MatchSkeleton } from "@/features/matches/components/match-skeleton";
import { Page } from "@/shared/ui/page";

/** A partida carregando: cabeçalho, placar, mapas e os blocos fechados. */
export default async function Loading() {
  const t = await getTranslations("Match");
  return (
    <Page width="wide">
      <MatchSkeleton label={t("loading")} />
    </Page>
  );
}
