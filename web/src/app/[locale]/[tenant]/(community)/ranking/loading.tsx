import { getTranslations } from "next-intl/server";

import { PageHeaderSkeleton, TableSkeleton } from "@/shared/ui/skeleton";

/** O Ranking carregando: o cabeçalho com o metadado e a tabela. */
export default async function Loading() {
  const t = await getTranslations("Ratings.ranking");
  return (
    <>
      <PageHeaderSkeleton breadcrumb="belowLg" meta label={t("loading")} />
      <TableSkeleton columns={6} />
    </>
  );
}
