import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { TenantTabSkeleton } from "@/features/tenants/components/tenant-tab-skeleton";
import { TournamentList } from "@/features/tournaments/components/tournament-list";
import { TableSkeleton } from "@/shared/ui/skeleton";

/** Aba Torneios da comunidade. */
export default async function TenantTournamentsPage({
  params,
}: PageProps<"/[locale]/[tenant]">) {
  const t = await getTranslations("Tournaments");
  return (
    <Suspense
      fallback={
        <TenantTabSkeleton label={t("loading")}>
          <TableSkeleton columns={5} />
        </TenantTabSkeleton>
      }
    >
      {params.then(({ tenant }) => (
        <TournamentList slug={tenant} />
      ))}
    </Suspense>
  );
}
