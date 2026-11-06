import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { TournamentSettings } from "@/features/tournaments/components/tournament-settings";
import { TournamentSettingsSkeleton } from "@/features/tournaments/components/tournament-settings-skeleton";

/** A aba Configurações. */
export default async function SettingsPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]/settings">) {
  const t = await getTranslations("TournamentSettings");
  return (
    <Suspense fallback={<TournamentSettingsSkeleton label={t("loading")} />}>
      {params.then(({ tenant, tournament }) => (
        <TournamentSettings tenant={tenant} slug={tournament} />
      ))}
    </Suspense>
  );
}
