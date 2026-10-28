import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { MatchSkeleton } from "@/features/matches/components/match-skeleton";
import { MatchView } from "@/features/matches/components/match-view";
import { LiveUpdates } from "@/features/realtime/components/live-updates";
import { Page } from "@/shared/ui/page";

export default async function MatchPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]/matches/[match]">) {
  const t = await getTranslations("Match");
  return (
    <Page width="wide">
      <Suspense fallback={<MatchSkeleton label={t("loading")} />}>
        {params.then(({ tenant, tournament, match }) => (
          <>
            {/* Só a própria partida (ou o torneio inteiro) recarrega. */}
            <LiveUpdates
              tenant={tenant}
              tournament={tournament}
              match={match}
            />
            <MatchView tenant={tenant} slug={tournament} matchKey={match} />
          </>
        ))}
      </Suspense>
    </Page>
  );
}
