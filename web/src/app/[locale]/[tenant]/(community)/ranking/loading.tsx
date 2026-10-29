import { getTranslations } from "next-intl/server";

import { TabLoading, TableSkeleton } from "@/shared/ui/skeleton";

/** A aba Ranking carregando (o cabeçalho e as abas já estão na tela). */
export default async function Loading() {
  const t = await getTranslations("Ratings.ranking");
  return (
    <TabLoading label={t("loading")}>
      <TableSkeleton columns={6} />
    </TabLoading>
  );
}
