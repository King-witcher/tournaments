import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { RankingTable } from "@/features/ratings/components/ranking-table";
import { TabLoading, TableSkeleton } from "@/shared/ui/skeleton";

/** Aba Ranking da comunidade. */
export default async function TenantRankingPage({
  params,
}: PageProps<"/[locale]/[tenant]/ranking">) {
  const t = await getTranslations("Ratings.ranking");
  return (
    <Suspense
      fallback={
        <TabLoading label={t("loading")}>
          <TableSkeleton columns={6} />
        </TabLoading>
      }
    >
      {params.then(({ tenant }) => (
        <RankingTable slug={tenant} />
      ))}
    </Suspense>
  );
}
