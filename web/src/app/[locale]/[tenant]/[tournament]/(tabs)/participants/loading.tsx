import { getTranslations } from "next-intl/server";

import { TabLoading } from "@/features/tournaments/components/tab-loading";
import { SectionSkeleton, TableSkeleton } from "@/shared/ui/skeleton";

/** Os inscritos carregando: o título e a tabela. */
export default async function Loading() {
  const t = await getTranslations("Participants");
  return (
    <TabLoading label={t("loading")}>
      <SectionSkeleton rows={0} />
      <TableSkeleton columns={4} />
    </TabLoading>
  );
}
