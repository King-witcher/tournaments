import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { TabLoading } from "@/features/tournaments/components/tab-loading";
import { TournamentRules } from "@/features/tournaments/components/tournament-rules";
import { SectionSkeleton } from "@/shared/ui/skeleton";

/** A aba Regras. */
export default async function RulesPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]/rules">) {
  const t = await getTranslations("Tournament");
  return (
    <Suspense
      fallback={
        <TabLoading label={t("loading")}>
          <SectionSkeleton rows={5} />
        </TabLoading>
      }
    >
      {params.then(({ tenant, tournament }) => (
        <TournamentRules tenant={tenant} slug={tournament} />
      ))}
    </Suspense>
  );
}
