import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { SeedingView } from "@/features/participants/components/seeding-view";
import { Page } from "@/shared/ui/page";
import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";

/** O seeding do torneio, em página inteira (fora das abas). */
export default async function SeedingPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]/seeding">) {
  const t = await getTranslations("Participants.seeding");
  return (
    <Page width="wide">
      <Suspense
        fallback={
          <>
            <PageHeaderSkeleton meta label={t("loading")} />
            <SectionSkeleton rows={6} />
          </>
        }
      >
        {params.then(({ tenant, tournament }) => (
          <SeedingView tenant={tenant} slug={tournament} />
        ))}
      </Suspense>
    </Page>
  );
}
