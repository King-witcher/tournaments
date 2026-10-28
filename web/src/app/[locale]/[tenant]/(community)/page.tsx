import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { TournamentList } from "@/features/tournaments/components/tournament-list";
import { TabLoading, TableSkeleton } from "@/shared/ui/skeleton";

/** Aba Torneios da comunidade. */
export default async function TenantTournamentsPage({
  params,
}: PageProps<"/[locale]/[tenant]">) {
  const t = await getTranslations("Tournaments");
  return (
    <Suspense
      fallback={
        <TabLoading label={t("loading")}>
          <TableSkeleton columns={5} />
        </TabLoading>
      }
    >
      {params.then(({ tenant }) => (
        <TournamentList slug={tenant} />
      ))}
    </Suspense>
  );
}
