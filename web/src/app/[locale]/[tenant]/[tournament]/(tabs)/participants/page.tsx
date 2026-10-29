import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { ParticipantsView } from "@/features/participants/components/participants-view";
import {
  SectionSkeleton,
  TabLoading,
  TableSkeleton,
} from "@/shared/ui/skeleton";

/** A aba Inscritos (no TDM, com os times e os reservas antes da tabela). */
export default async function ParticipantsPage({
  params,
}: PageProps<"/[locale]/[tenant]/[tournament]/participants">) {
  const t = await getTranslations("Participants");
  return (
    <Suspense
      fallback={
        <TabLoading label={t("loading")}>
          <SectionSkeleton rows={0} />
          <TableSkeleton columns={4} />
        </TabLoading>
      }
    >
      {params.then(({ tenant, tournament }) => (
        <ParticipantsView tenant={tenant} slug={tournament} />
      ))}
    </Suspense>
  );
}
