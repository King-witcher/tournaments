import { getTranslations } from "next-intl/server";

import { TabLoading, TableSkeleton } from "@/shared/ui/skeleton";

/** Os torneios carregando, dentro da casca. */
export default async function Loading() {
  const t = await getTranslations("Member.loadingTab");
  return (
    <TabLoading label={t("tournaments")}>
      <TableSkeleton columns={4} rows={5} />
    </TabLoading>
  );
}
