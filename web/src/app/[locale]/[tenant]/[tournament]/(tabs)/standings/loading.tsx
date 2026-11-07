import { getTranslations } from "next-intl/server";

import { TabLoading, TableSkeleton } from "@/shared/ui/skeleton";

/** A classificação carregando: a tabela (o título é o da página). */
export default async function Loading() {
  const t = await getTranslations("Bracket.standings");
  return (
    <TabLoading label={t("loading")}>
      <TableSkeleton columns={5} />
    </TabLoading>
  );
}
