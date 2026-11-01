import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { TournamentRules } from "@/features/tournaments/components/tournament-rules";
import { TournamentRulesSkeleton } from "@/features/tournaments/components/tournament-rules-skeleton";

/** A aba Regras. */
export default async function RulesPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]/rules">) {
  const t = await getTranslations("Tournament");
  return (
    <Suspense fallback={<TournamentRulesSkeleton label={t("loading")} />}>
      {params.then(({ tenant, tournament }) => (
        <TournamentRules tenant={tenant} slug={tournament} />
      ))}
    </Suspense>
  );
}
