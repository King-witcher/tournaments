import { getTranslations } from "next-intl/server";

import { TournamentRulesSkeleton } from "@/features/tournaments/components/tournament-rules-skeleton";

/** As regras carregando: o texto gerado do formato e a grade dos mapas. */
export default async function Loading() {
  const t = await getTranslations("Tournament");
  return <TournamentRulesSkeleton label={t("loading")} />;
}
