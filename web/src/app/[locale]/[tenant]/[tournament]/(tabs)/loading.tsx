import { getTranslations } from "next-intl/server";

import { TournamentOverviewSkeleton } from "@/features/tournaments/components/tournament-overview-skeleton";

/** A Visão geral carregando. */
export default async function Loading() {
  const t = await getTranslations("Tournament");
  return <TournamentOverviewSkeleton label={t("loading")} />;
}
