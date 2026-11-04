import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { BracketTab } from "@/features/matches/components/bracket-tab";
import {
  SectionSkeleton,
  TabLoading,
  TableSkeleton,
} from "@/shared/ui/skeleton";

/**
 * A aba Classificação: só o conteúdo da aba (o cabeçalho do torneio e as abas
 * vêm do `layout.tsx` do grupo).
 */
export default async function StandingsPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]/standings">) {
  const t = await getTranslations("Bracket.standings");
  return (
    <Suspense
      fallback={
        <TabLoading label={t("loading")}>
          <SectionSkeleton rows={0} />
          <TableSkeleton columns={5} />
        </TabLoading>
      }
    >
      {params.then(({ tenant, tournament }) => (
        <BracketTab tenant={tenant} slug={tournament} view="standings" />
      ))}
    </Suspense>
  );
}
