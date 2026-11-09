import { getTranslations } from "next-intl/server";
import { Suspense } from "react";
import { FormationSkeleton } from "@/features/teams/components/formation-skeleton";
import { TeamsView } from "@/features/teams/components/teams-view";

/** A seção Times, nos torneios de times. */
export default async function TeamsPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]/teams">) {
  const t = await getTranslations("Team");
  return (
    <Suspense fallback={<FormationSkeleton label={t("loading")} />}>
      {params.then(({ tenant, tournament }) => (
        <TeamsView tenant={tenant} slug={tournament} />
      ))}
    </Suspense>
  );
}
