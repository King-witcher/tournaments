import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { EditTournamentView } from "@/features/tournaments/components/edit-tournament-view";
import { Page } from "@/shared/ui/page";

export default async function EditTournamentPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]/edit">) {
  const t = await getTranslations("Tournament");
  return (
    <Page spacing="loose">
      <Suspense fallback={<p>{t("loading")}</p>}>
        {params.then(({ tenant, tournament }) => (
          <EditTournamentView tenant={tenant} slug={tournament} />
        ))}
      </Suspense>
    </Page>
  );
}
