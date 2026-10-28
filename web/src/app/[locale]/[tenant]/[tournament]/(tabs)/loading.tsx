import { getTranslations } from "next-intl/server";

import { SectionSkeleton, TabLoading } from "@/shared/ui/skeleton";

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
