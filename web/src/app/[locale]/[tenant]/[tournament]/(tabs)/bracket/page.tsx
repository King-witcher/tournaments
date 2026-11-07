import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { BracketSkeleton } from "@/features/matches/components/bracket-skeleton";
import { BracketTab } from "@/features/matches/components/bracket-tab";
import { parseStageParam } from "@/features/stages/lib/stages";

/**
 * A Chave, na largura toda (o cabeçalho vem do `layout.tsx` das seções).
 */
export default async function BracketPage({
  params,
  searchParams,
}: PageProps<"/[locale]/[tenant]/[tournament]/bracket">) {
  const t = await getTranslations("Bracket");
  return (
    <Suspense fallback={<BracketSkeleton label={t("loading")} />}>
      {Promise.all([params, searchParams]).then(
        ([{ tenant, tournament }, { stage }]) => (
          <BracketTab
            tenant={tenant}
            slug={tournament}
            view="bracket"
            stage={parseStageParam(stage)}
          />
        ),
      )}
    </Suspense>
  );
}
