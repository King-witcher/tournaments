import { getTranslations } from "next-intl/server";

import { FormationSkeleton } from "@/features/teams/components/formation-skeleton";
import { Page } from "@/shared/ui/page";

/** Os times carregando: a trilha, o título com a explicação, o resumo e a formação. */
export default async function Loading() {
  const t = await getTranslations("Team");
  return (
    <Page width="wide">
      <FormationSkeleton label={t("loading")} />
    </Page>
  );
}
