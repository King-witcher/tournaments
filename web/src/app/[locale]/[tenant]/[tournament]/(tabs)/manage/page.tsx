import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { TabLoading } from "@/features/tournaments/components/tab-loading";
import { TournamentManage } from "@/features/tournaments/components/tournament-manage";
import { SectionSkeleton } from "@/shared/ui/skeleton";

/** A aba Organização. */
export default async function ManagePage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]/manage">) {
  const t = await getTranslations("Tournament");
  return (
    <Suspense
      fallback={
        <TabLoading label={t("loading")}>
          <SectionSkeleton rows={2} />
          <SectionSkeleton rows={1} />
          <SectionSkeleton rows={2} />
        </TabLoading>
      }
    >
      {params.then(({ tenant, tournament }) => (
        <TournamentManage tenant={tenant} slug={tournament} />
      ))}
    </Suspense>
  );
}
