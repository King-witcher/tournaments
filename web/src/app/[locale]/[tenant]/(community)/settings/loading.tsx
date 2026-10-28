import { getTranslations } from "next-intl/server";

import { SectionSkeleton, TabLoading } from "@/shared/ui/skeleton";

/** A aba Configurações carregando (o cabeçalho e as abas já estão na tela). */
export default async function Loading() {
  const t = await getTranslations("Tenant.settings");
  return (
    <TabLoading label={t("loading")}>
      <SectionSkeleton rows={2} />
      <SectionSkeleton rows={1} />
    </TabLoading>
  );
}
