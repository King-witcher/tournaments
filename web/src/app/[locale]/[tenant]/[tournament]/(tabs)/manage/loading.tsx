import { getTranslations } from "next-intl/server";

import { SectionSkeleton, TabLoading } from "@/shared/ui/skeleton";

/** A organização carregando: o estado, o seeding e os links de inscrição. */
export default async function Loading() {
  const t = await getTranslations("Tournament");
  return (
    <TabLoading label={t("loading")}>
      <SectionSkeleton rows={2} />
      <SectionSkeleton rows={1} />
      <SectionSkeleton rows={2} />
    </TabLoading>
  );
}
