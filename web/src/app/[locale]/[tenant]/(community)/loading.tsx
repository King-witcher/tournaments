import { getTranslations } from "next-intl/server";

import { TenantTabSkeleton } from "@/features/tenants/components/tenant-tab-skeleton";
import { TableSkeleton } from "@/shared/ui/skeleton";

/** A aba Torneios carregando (o cabeçalho e as abas já estão na tela). */
export default async function Loading() {
  const t = await getTranslations("Tournaments");
  return (
    <TenantTabSkeleton label={t("loading")}>
      <TableSkeleton columns={5} />
    </TenantTabSkeleton>
  );
}
