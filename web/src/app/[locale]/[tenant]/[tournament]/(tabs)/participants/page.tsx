import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { ParticipantTable } from "@/features/participants/components/participant-table";
import {
  SectionSkeleton,
  TabLoading,
  TableSkeleton,
} from "@/shared/ui/skeleton";

/** A aba Inscritos. */
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
        <ParticipantTable tenant={tenant} slug={tournament} />
      ))}
    </Suspense>
  );
}
