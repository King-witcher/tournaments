import { getTranslations } from "next-intl/server";

import { Page } from "@/shared/ui/page";
import { FormSkeleton, PageHeaderSkeleton } from "@/shared/ui/skeleton";

/** Criar torneio carregando: a trilha, o título e os quatro campos. */
export default async function Loading() {
  const t = await getTranslations("NewTournament");
  return (
    <Page>
      <PageHeaderSkeleton label={t("loading")} />
      <FormSkeleton fields={4} label={false} />
    </Page>
  );
}
