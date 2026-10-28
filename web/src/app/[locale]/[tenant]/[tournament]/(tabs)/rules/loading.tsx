import { getTranslations } from "next-intl/server";

import { SectionSkeleton, TabLoading } from "@/shared/ui/skeleton";

/** As regras carregando: o texto. */
export default async function Loading() {
  const t = await getTranslations("Tournament");
  return (
    <TabLoading label={t("loading")}>
      <SectionSkeleton rows={5} />
    </TabLoading>
  );
}
