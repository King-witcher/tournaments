import { getTranslations } from "next-intl/server";

import { SettingsSectionSkeleton } from "@/features/tournaments/components/settings-section-skeleton";

/** Uma seção das Configurações carregando, dentro do layout delas. */
export default async function Loading() {
  const t = await getTranslations("TournamentSettings");
  return <SettingsSectionSkeleton label={t("loading")} />;
}
