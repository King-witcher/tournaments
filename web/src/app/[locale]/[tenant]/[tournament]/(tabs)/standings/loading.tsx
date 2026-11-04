import { getTranslations } from "next-intl/server";

import {
  SectionSkeleton,
  TabLoading,
  TableSkeleton,
} from "@/shared/ui/skeleton";

/** A classificação carregando: o título e a tabela. */
export default async function Loading() {
  const t = await getTranslations("Bracket.standings");
  return (
    <TabLoading label={t("loading")}>
      <SectionSkeleton rows={0} />
      <TableSkeleton columns={5} />
    </TabLoading>
  );
}
