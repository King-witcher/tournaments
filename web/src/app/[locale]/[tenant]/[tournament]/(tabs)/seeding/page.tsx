import { getTranslations } from "next-intl/server";
import { Suspense } from "react";
import { SeedingView } from "@/features/teams/components/seeding-view";
import { SectionSkeleton, TabLoading } from "@/shared/ui/skeleton";

/** A seção Ordem da chave, para a organização. */
export default async function SeedingPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]/seeding">) {
  const t = await getTranslations("Participants.seeding");
  return (
    <Suspense
      fallback={
        <TabLoading label={t("loading")}>
          <SectionSkeleton rows={6} />
        </TabLoading>
      }
    >
      {params.then(({ tenant, tournament }) => (
        <SeedingView tenant={tenant} slug={tournament} />
      ))}
    </Suspense>
  );
}
