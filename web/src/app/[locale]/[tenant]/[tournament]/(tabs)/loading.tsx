import { getTranslations } from "next-intl/server";

import { TabLoading } from "@/features/tournaments/components/tab-loading";
import { SectionSkeleton } from "@/shared/ui/skeleton";

/** A Visão geral carregando: o ciclo, a descrição e o formato. */
export default async function Loading() {
  const t = await getTranslations("Tournament");
  return (
    <TabLoading label={t("loading")}>
      <SectionSkeleton rows={1} />
      <SectionSkeleton rows={4} />
    </TabLoading>
  );
}
