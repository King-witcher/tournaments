import { getTranslations } from "next-intl/server";

import { SectionSkeleton, TabLoading } from "@/shared/ui/skeleton";

/** A ordem da chave carregando: o título e a lista. */
export default async function Loading() {
  const t = await getTranslations("Participants.seeding");
  return (
    <TabLoading label={t("loading")}>
      <SectionSkeleton rows={6} />
    </TabLoading>
  );
}
