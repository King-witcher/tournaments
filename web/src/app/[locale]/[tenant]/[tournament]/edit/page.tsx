import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { EditTournamentView } from "@/features/tournaments/components/edit-tournament-view";
import { Page } from "@/shared/ui/page";
import { PageSkeleton } from "@/shared/ui/skeleton";

export default async function EditTournamentPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]/edit">) {
  const t = await getTranslations("Tournament");
  return (
    <Page spacing="loose">
      <Suspense fallback={<PageSkeleton label={t("loading")} />}>
        {params.then(({ tenant, tournament }) => (
          <EditTournamentView tenant={tenant} slug={tournament} />
        ))}
      </Suspense>
    </Page>
  );
}
