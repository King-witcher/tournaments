import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { BracketSkeleton } from "@/features/matches/components/bracket-skeleton";
import { BracketTab } from "@/features/matches/components/bracket-tab";

/**
 * A aba Chave: só o conteúdo da aba (o cabeçalho do torneio e as abas vêm
 * do `layout.tsx` do grupo).
 */
export default async function BracketPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]/bracket">) {
  const t = await getTranslations("Bracket");
  return (
    <Suspense fallback={<BracketSkeleton label={t("loading")} />}>
      {params.then(({ tenant, tournament }) => (
        <BracketTab tenant={tenant} slug={tournament} />
      ))}
    </Suspense>
  );
}
