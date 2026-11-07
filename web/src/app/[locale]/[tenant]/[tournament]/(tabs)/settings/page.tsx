import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { SettingsSectionSkeleton } from "@/features/tournaments/components/settings-section-skeleton";
import { TournamentSettingsSection } from "@/features/tournaments/components/tournament-settings-section";

/** As Configurações abrem em Geral. */
export default async function SettingsPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]/settings">) {
  const t = await getTranslations("TournamentSettings");
  return (
    <Suspense fallback={<SettingsSectionSkeleton label={t("loading")} />}>
      {params.then(({ tenant, tournament }) => (
        <TournamentSettingsSection
          tenant={tenant}
          slug={tournament}
          section="general"
        />
      ))}
    </Suspense>
  );
}
