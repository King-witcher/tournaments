import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { MatchView } from "@/features/matches/components/match-view";
import { Page } from "@/shared/ui/page";

export default async function MatchPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]/matches/[match]">) {
  const t = await getTranslations("Match");
  return (
    <Page spacing="loose">
      <Suspense fallback={<p>{t("loading")}</p>}>
        {params.then(({ tenant, tournament, match }) => (
          <MatchView tenant={tenant} slug={tournament} matchKey={match} />
        ))}
      </Suspense>
    </Page>
  );
}
