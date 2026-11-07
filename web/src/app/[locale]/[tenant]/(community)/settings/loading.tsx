import { getTranslations } from "next-intl/server";

import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";

/** As Configurações carregando: o cabeçalho e as primeiras seções. */
export default async function Loading() {
  const t = await getTranslations("Tenant.settings");
  return (
    <>
      <PageHeaderSkeleton label={t("loading")} />
      <SectionSkeleton rows={2} />
      <SectionSkeleton rows={1} />
    </>
  );
}
