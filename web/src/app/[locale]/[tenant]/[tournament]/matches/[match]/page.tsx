import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { MatchSkeleton } from "@/features/matches/components/match-skeleton";
import { MatchView } from "@/features/matches/components/match-view";
import { Page } from "@/shared/ui/page";

export default async function MatchPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]/matches/[match]">) {
  const t = await getTranslations("Match");
  return (
    <Page width="wide">
      <Suspense fallback={<MatchSkeleton label={t("loading")} />}>
        {params.then(({ tenant, tournament, match }) => (
          <MatchView tenant={tenant} slug={tournament} matchKey={match} />
        ))}
      </Suspense>
    </Page>
  );
}
