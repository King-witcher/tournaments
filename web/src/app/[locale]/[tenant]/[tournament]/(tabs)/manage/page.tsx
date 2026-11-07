import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { TournamentManage } from "@/features/tournaments/components/tournament-manage";
import { TournamentManageSkeleton } from "@/features/tournaments/components/tournament-manage-skeleton";

/** A Organização (o cabeçalho vem do `layout.tsx` das seções). */
export default async function ManagePage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]/manage">) {
  const t = await getTranslations("Tournament");
  return (
    <Suspense fallback={<TournamentManageSkeleton label={t("loading")} />}>
      {params.then(({ tenant, tournament }) => (
        <TournamentManage tenant={tenant} slug={tournament} />
      ))}
    </Suspense>
  );
}
