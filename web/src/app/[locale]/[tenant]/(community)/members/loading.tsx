import { getTranslations } from "next-intl/server";

import { TenantTabSkeleton } from "@/features/tenants/components/tenant-tab-skeleton";
import { TableSkeleton } from "@/shared/ui/skeleton";

/** A aba Membros carregando (o cabeçalho e as abas já estão na tela). */
export default async function Loading() {
  const t = await getTranslations("Tenant.members");
  return (
    <TenantTabSkeleton label={t("loading")}>
      <TableSkeleton columns={3} />
    </TenantTabSkeleton>
  );
}
