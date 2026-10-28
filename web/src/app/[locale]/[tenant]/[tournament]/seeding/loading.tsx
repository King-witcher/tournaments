import { getTranslations } from "next-intl/server";

import { Page } from "@/shared/ui/page";
import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";

/** O seeding carregando: a trilha, o título com a explicação e a lista. */
export default async function Loading() {
  const t = await getTranslations("Participants.seeding");
  return (
    <Page width="wide">
      <PageHeaderSkeleton meta label={t("loading")} />
      <SectionSkeleton rows={6} />
    </Page>
  );
}
