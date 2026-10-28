import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { NewTournamentView } from "@/features/tournaments/components/new-tournament-view";
import { Page } from "@/shared/ui/page";
import { PageSkeleton } from "@/shared/ui/skeleton";

export default async function NewTournamentPage({
  params,
}: PageProps<"/[locale]/[tenant]/new">) {
  const t = await getTranslations("Tenant");
  return (
    <Page spacing="loose">
      <Suspense fallback={<PageSkeleton label={t("loading")} />}>
        {params.then(({ tenant }) => (
          <NewTournamentView tenant={tenant} />
        ))}
      </Suspense>
    </Page>
  );
}
