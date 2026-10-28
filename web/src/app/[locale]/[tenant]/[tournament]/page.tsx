import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { TournamentView } from "@/features/tournaments/components/tournament-view";
import { Page } from "@/shared/ui/page";
import { PageSkeleton } from "@/shared/ui/skeleton";

export default async function TournamentPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]">) {
  const t = await getTranslations("Tournament");
  return (
    <Page spacing="loose">
      <Suspense fallback={<PageSkeleton label={t("loading")} />}>
        {params.then(({ tenant, tournament }) => (
          <TournamentView tenant={tenant} slug={tournament} />
        ))}
      </Suspense>
    </Page>
  );
}
