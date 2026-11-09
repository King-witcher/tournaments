import { getTranslations } from "next-intl/server";

import { FormationSkeleton } from "@/features/teams/components/formation-skeleton";

/** Os times carregando: a grade dos times e os reservas. */
export default async function Loading() {
  const t = await getTranslations("Team");
  return <FormationSkeleton label={t("loading")} />;
}
