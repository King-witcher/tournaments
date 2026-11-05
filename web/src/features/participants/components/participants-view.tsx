import { Suspense } from "react";

import { TeamsSection } from "@/features/teams/components/teams-section";
import { loadTournament } from "@/features/tournaments/server";
import { preload } from "@/shared/lib/preload";
import { Stack } from "@/shared/ui/carbon/stack";
import { SectionSkeleton, TableSkeleton } from "@/shared/ui/skeleton";
import { getParticipants } from "../server";
import { ParticipantTable } from "./participant-table";

type Props = { tenant: string; slug: string };

/**
 * A aba Inscritos. No TDM, primeiro os times e os reservas (para todos),
 * depois a tabela das inscrições individuais (check-in, tirar alguém); no
 * duelo, a tabela. Fica em `<Suspense>`.
 *
 * A tabela sai em todo modo: os inscritos começam junto com o torneio, não
 * depois dele.
 */
export async function ParticipantsView({ tenant, slug }: Props) {
  preload(getParticipants(tenant, slug));
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
