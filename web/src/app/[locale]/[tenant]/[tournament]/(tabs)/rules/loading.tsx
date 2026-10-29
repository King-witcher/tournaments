import { getTranslations } from "next-intl/server";

import { SectionSkeleton, TabLoading } from "@/shared/ui/skeleton";

/** As regras carregando: o texto gerado do formato, com uns oito parágrafos. */
export default async function Loading() {
  const t = await getTranslations("Tournament");
  return (
    <TabLoading label={t("loading")}>
      <SectionSkeleton rows={8} />
    </TabLoading>
  );
}
