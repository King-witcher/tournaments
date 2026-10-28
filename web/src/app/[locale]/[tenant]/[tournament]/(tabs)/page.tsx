import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { TournamentOverview } from "@/features/tournaments/components/tournament-overview";
import { SectionSkeleton, TabLoading } from "@/shared/ui/skeleton";

/** A aba Visão geral (o cabeçalho e as abas vêm do `layout.tsx` do grupo). */
export default async function TournamentOverviewPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]">) {
  const t = await getTranslations("Tournament");
  return (
    <Suspense
      fallback={
        <TabLoading label={t("loading")}>
          <SectionSkeleton rows={1} />
          <SectionSkeleton rows={4} />
        </TabLoading>
      }
    >
      {params.then(({ tenant, tournament }) => (
        <TournamentOverview tenant={tenant} slug={tournament} />
      ))}
    </Suspense>
  );
}
