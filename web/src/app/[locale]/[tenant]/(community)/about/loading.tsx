import { getTranslations } from "next-intl/server";

import { SectionSkeleton, TabLoading } from "@/shared/ui/skeleton";

/** A aba Sobre carregando (o cabeçalho e as abas já estão na tela). */
export default async function Loading() {
  const t = await getTranslations("Tenant.about");
  return (
    <TabLoading label={t("loading")}>
      <SectionSkeleton rows={3} />
    </TabLoading>
  );
}
