import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { NewTournamentView } from "@/features/tournaments/components/new-tournament-view";
import { TournamentFormSkeleton } from "@/features/tournaments/components/tournament-form-skeleton";
import { Page } from "@/shared/ui/page";

export default async function NewTournamentPage({
  params,
  searchParams,
}: PageProps<"/[locale]/[tenant]/new">) {
  const t = await getTranslations("NewTournament");
  return (
    <Page>
      <Suspense fallback={<TournamentFormSkeleton label={t("loading")} />}>
        {Promise.all([params, searchParams]).then(([{ tenant }, { from }]) => (
          <NewTournamentView
            tenant={tenant}
            from={typeof from === "string" ? from : undefined}
          />
        ))}
      </Suspense>
    </Page>
  );
}
