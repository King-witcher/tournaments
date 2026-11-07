import { getTranslations } from "next-intl/server";

import { TournamentManageSkeleton } from "@/features/tournaments/components/tournament-manage-skeleton";

/** A Organização carregando. */
export default async function Loading() {
  const t = await getTranslations("Tournament");
  return <TournamentManageSkeleton label={t("loading")} />;
}
