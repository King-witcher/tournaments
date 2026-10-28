import { getTranslations } from "next-intl/server";

import { TabLoading } from "@/features/tournaments/components/tab-loading";
import { SectionSkeleton } from "@/shared/ui/skeleton";

/** As regras carregando: o texto. */
export default async function Loading() {
  const t = await getTranslations("Tournament");
  return (
    <TabLoading label={t("loading")}>
      <SectionSkeleton rows={5} />
    </TabLoading>
  );
}
