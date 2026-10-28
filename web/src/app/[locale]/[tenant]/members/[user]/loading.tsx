import { getTranslations } from "next-intl/server";

import { Page } from "@/shared/ui/page";
import { PageHeaderSkeleton, TableSkeleton } from "@/shared/ui/skeleton";

/** A página da pessoa carregando: cabeçalho e os torneios dela. */
export default async function Loading() {
  const t = await getTranslations("Member");
  return (
    <Page width="wide">
      <PageHeaderSkeleton meta label={t("loading")} />
      <TableSkeleton columns={4} rows={3} />
    </Page>
  );
}
