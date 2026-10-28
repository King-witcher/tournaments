import { getTranslations } from "next-intl/server";
import { Suspense } from "react";
import { LiveUpdates } from "@/features/realtime/components/live-updates";
import { EditTournamentView } from "@/features/tournaments/components/edit-tournament-view";
import { TournamentFormSkeleton } from "@/features/tournaments/components/tournament-form-skeleton";
import { Page } from "@/shared/ui/page";

export default async function EditTournamentPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]/edit">) {
  const t = await getTranslations("TournamentForm.edit");
  return (
    <Page>
      <Suspense fallback={<TournamentFormSkeleton label={t("loading")} edit />}>
        {params.then(({ tenant, tournament }) => (
          <>
            {/* O que já foi digitado fica: a recarga só troca o que veio do servidor. */}
            <LiveUpdates tenant={tenant} tournament={tournament} />
            <EditTournamentView tenant={tenant} slug={tournament} />
          </>
        ))}
      </Suspense>
    </Page>
  );
}
