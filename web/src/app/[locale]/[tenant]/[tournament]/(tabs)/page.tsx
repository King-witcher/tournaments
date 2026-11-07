import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { TournamentOverview } from "@/features/tournaments/components/tournament-overview";
import { TournamentOverviewSkeleton } from "@/features/tournaments/components/tournament-overview-skeleton";

/** A Visão geral (o cabeçalho vem do `layout.tsx` das seções). */
export default async function TournamentOverviewPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]">) {
  const t = await getTranslations("Tournament");
  return (
    <Suspense fallback={<TournamentOverviewSkeleton label={t("loading")} />}>
      {params.then(({ tenant, tournament }) => (
        <TournamentOverview tenant={tenant} slug={tournament} />
      ))}
    </Suspense>
  );
}
