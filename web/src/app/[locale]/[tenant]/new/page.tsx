import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { NewTournamentView } from "@/features/tournaments/components/new-tournament-view";
import { Page } from "@/shared/ui/page";

export default async function NewTournamentPage({
  params,
}: PageProps<"/[locale]/[tenant]/new">) {
  const t = await getTranslations("Tenant");
  return (
    <Page spacing="loose">
      <Suspense fallback={<p>{t("loading")}</p>}>
        {params.then(({ tenant }) => (
          <NewTournamentView tenant={tenant} />
        ))}
      </Suspense>
    </Page>
  );
}
