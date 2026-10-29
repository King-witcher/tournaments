import { Stack } from "@carbon/react";
import { Suspense } from "react";

import { TeamsSection } from "@/features/teams/components/teams-section";
import { loadTournament } from "@/features/tournaments/server";
import { SectionSkeleton, TableSkeleton } from "@/shared/ui/skeleton";
import { ParticipantTable } from "./participant-table";

type Props = { tenant: string; slug: string };

/**
 * A aba Inscritos. No TDM, primeiro os times e os reservas (para todos),
 * depois a tabela das inscrições individuais (check-in, tirar alguém); no
 * duelo, só a tabela. Fica em `<Suspense>`.
 */
export async function ParticipantsView({ tenant, slug }: Props) {
  const loaded = await loadTournament(tenant, slug);
  if (loaded.kind !== "ok") return null;

  const table = <ParticipantTable tenant={tenant} slug={slug} />;
  if (loaded.tournament.mode !== "tdm") return table;
  return (
    <Stack gap={7}>
      <Suspense fallback={<SectionSkeleton tiles={2} />}>
        <TeamsSection tenant={tenant} slug={slug} />
      </Suspense>
      <Suspense
        fallback={
          <>
            <SectionSkeleton rows={0} />
            <TableSkeleton columns={4} />
          </>
        }
      >
        {table}
      </Suspense>
    </Stack>
  );
}
