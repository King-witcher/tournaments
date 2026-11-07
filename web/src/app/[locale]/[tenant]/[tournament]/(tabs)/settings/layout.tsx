import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { TournamentSettings } from "@/features/tournaments/components/tournament-settings";
import { TournamentSettingsSkeleton } from "@/features/tournaments/components/tournament-settings-skeleton";

/**
 * As Configurações do torneio: o estado delas (o salvo e o que não foi salvo
 * de cada seção) em volta da página da seção, que troca pelo menu sem
 * remontar este layout.
 */
export default async function SettingsLayout({
  params,
  children,
}: LayoutProps<"/[locale]/[tenant]/[tournament]/settings">) {
  const t = await getTranslations("TournamentSettings");
  return (
    <Suspense fallback={<TournamentSettingsSkeleton label={t("loading")} />}>
      {params.then(({ tenant, tournament }) => (
        <TournamentSettings tenant={tenant} slug={tournament}>
          {children}
        </TournamentSettings>
      ))}
    </Suspense>
  );
}
