import { getTranslations } from "next-intl/server";

import { PageHeaderSkeleton, TableSkeleton } from "@/shared/ui/skeleton";

/** Membros carregando: o cabeçalho com o metadado e a tabela. */
export default async function Loading() {
  const t = await getTranslations("Tenant.members");
  return (
    <>
      <PageHeaderSkeleton breadcrumb="belowLg" meta label={t("loading")} />
      <TableSkeleton columns={3} />
    </>
  );
}
