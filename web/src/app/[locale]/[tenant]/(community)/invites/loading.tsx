import { getTranslations } from "next-intl/server";

import { TabLoading, TableSkeleton } from "@/shared/ui/skeleton";

/** A aba Convites carregando (o cabeçalho e as abas já estão na tela). */
export default async function Loading() {
  const t = await getTranslations("Tenant.invites");
  return (
    <TabLoading label={t("loading")}>
      <TableSkeleton columns={4} rows={3} />
    </TabLoading>
  );
}
