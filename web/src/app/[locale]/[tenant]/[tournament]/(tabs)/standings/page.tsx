import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { BracketTab } from "@/features/matches/components/bracket-tab";
import { parseStageParam } from "@/features/stages/lib/stages";
import { TabLoading, TableSkeleton } from "@/shared/ui/skeleton";

/**
 * A Classificação (o cabeçalho vem do `layout.tsx` das seções).
 */
export default async function StandingsPage({
  params,
  searchParams,
}: PageProps<"/[locale]/[tenant]/[tournament]/standings">) {
  const t = await getTranslations("Bracket.standings");
  return (
    <Suspense
      fallback={
        <TabLoading label={t("loading")}>
          <TableSkeleton columns={5} />
        </TabLoading>
      }
    >
      {Promise.all([params, searchParams]).then(
        ([{ tenant, tournament }, { stage }]) => (
          <BracketTab
            tenant={tenant}
            slug={tournament}
            view="standings"
            stage={parseStageParam(stage)}
          />
        ),
      )}
    </Suspense>
  );
}
