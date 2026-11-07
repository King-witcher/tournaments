import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { RankingMeta } from "@/features/ratings/components/ranking-meta";
import { RankingTable } from "@/features/ratings/components/ranking-table";
import { TenantHeader } from "@/features/tenants/components/tenant-header";
import { PageHeaderSkeleton, TableSkeleton } from "@/shared/ui/skeleton";

/** O Ranking da comunidade: o cabeçalho com o rating inicial e a tabela. */
export default async function TenantRankingPage({
  params,
}: PageProps<"/[locale]/[tenant]/ranking">) {
  const [t, tRanking] = await Promise.all([
    getTranslations("Tenant"),
    getTranslations("Ratings.ranking"),
  ]);
  return (
    <>
      <Suspense
        fallback={<PageHeaderSkeleton meta label={tRanking("loading")} />}
      >
        {params.then(({ tenant }) => (
          <TenantHeader
            slug={tenant}
            title={t("tabs.ranking")}
            meta={<RankingMeta slug={tenant} />}
          />
        ))}
      </Suspense>
      <Suspense fallback={<TableSkeleton columns={6} />}>
        {params.then(({ tenant }) => (
          <RankingTable slug={tenant} />
        ))}
      </Suspense>
    </>
  );
}
