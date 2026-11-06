import { getTranslations } from "next-intl/server";

import { TournamentSettingsSkeleton } from "@/features/tournaments/components/tournament-settings-skeleton";

/** As configurações carregando: as primeiras seções, na coluna estreita. */
export default async function Loading() {
  const t = await getTranslations("TournamentSettings");
  return <TournamentSettingsSkeleton label={t("loading")} />;
}
