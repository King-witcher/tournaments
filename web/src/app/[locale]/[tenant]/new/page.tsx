import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { NewTournamentView } from "@/features/tournaments/components/new-tournament-view";
import { Page } from "@/shared/ui/page";
import { FormSkeleton, PageHeaderSkeleton } from "@/shared/ui/skeleton";

export default async function NewTournamentPage({
  params,
  searchParams,
}: PageProps<"/[locale]/[tenant]/new">) {
  const t = await getTranslations("NewTournament");
  return (
    <Page>
      <Suspense
        fallback={
          <>
            <PageHeaderSkeleton label={t("loading")} />
            <FormSkeleton fields={4} label={false} />
          </>
        }
      >
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
