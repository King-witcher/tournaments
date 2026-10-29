import { getTranslations } from "next-intl/server";
import { Suspense } from "react";
import { LiveUpdates } from "@/features/realtime/components/live-updates";
import { FormationSkeleton } from "@/features/teams/components/formation-skeleton";
import { TeamsView } from "@/features/teams/components/teams-view";
import { Page } from "@/shared/ui/page";

/** Os times do TDM (a formação), em página inteira (fora das abas). */
export default async function TeamsPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]/teams">) {
  const t = await getTranslations("Team");
  return (
    <Page width="wide">
      <Suspense fallback={<FormationSkeleton label={t("loading")} />}>
        {params.then(({ tenant, tournament }) => (
          <>
            <LiveUpdates tenant={tenant} tournament={tournament} />
            <TeamsView tenant={tenant} slug={tournament} />
          </>
        ))}
      </Suspense>
    </Page>
  );
}
