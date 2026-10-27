import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { TournamentView } from "@/features/tournaments/components/tournament-view";
import { Page } from "@/shared/ui/page";

export default async function TournamentPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]">) {
  const t = await getTranslations("Tournament");
  return (
    <Page spacing="loose">
      <Suspense fallback={<p>{t("loading")}</p>}>
        {params.then(({ tenant, tournament }) => (
          <TournamentView tenant={tenant} slug={tournament} />
        ))}
      </Suspense>
    </Page>
  );
}
