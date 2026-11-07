import { notFound } from "next/navigation";
import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { SettingsSectionSkeleton } from "@/features/tournaments/components/settings-section-skeleton";
import { TournamentSettingsSection } from "@/features/tournaments/components/tournament-settings-section";
import { sectionOfSegment } from "@/features/tournaments/lib/settings-sections";

/**
 * Uma seção das Configurações fora Geral (`/settings/maps`); Geral é a raiz
 * delas, e `/settings/general` não existe.
 */
export default async function SettingsSectionPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]/settings/[section]">) {
  const t = await getTranslations("TournamentSettings");
  return (
    <Suspense fallback={<SettingsSectionSkeleton label={t("loading")} />}>
      {params.then(({ tenant, tournament, section }) => {
        const key = sectionOfSegment(section);
        if (!key) notFound();
        return (
          <TournamentSettingsSection
            tenant={tenant}
            slug={tournament}
            section={key}
          />
        );
      })}
    </Suspense>
  );
}
