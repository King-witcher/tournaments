import { Stack } from "@carbon/react";
import { Suspense } from "react";

import { TournamentInviteSection } from "@/features/invites/components/tournament-invite-section";
import { TeamsSection } from "@/features/teams/components/teams-section";
import { invitesAllowed } from "@/features/tournaments/lib/status";
import { loadTournament } from "@/features/tournaments/server";
import { SectionSkeleton, TableSkeleton } from "@/shared/ui/skeleton";
import { ParticipantTable } from "./participant-table";

type Props = { tenant: string; slug: string };

/**
 * A aba Inscritos. No TDM, primeiro os times e os reservas (para todos),
 * depois a tabela das inscrições individuais (check-in, tirar alguém); no
 * duelo, a tabela. Para a organização, com o rascunho ou a inscrição aberta,
 * os links de inscrição depois da tabela. Fica em `<Suspense>`.
 */
export async function ParticipantsView({ tenant, slug }: Props) {
  const loaded = await loadTournament(tenant, slug);
  if (loaded.kind !== "ok") return null;

  const { me, tournament } = loaded;
  const invites =
    me.role !== "player" && invitesAllowed(tournament.status) ? (
      <Suspense fallback={<SectionSkeleton rows={2} />}>
        <TournamentInviteSection tenant={tenant} tournament={tournament} />
      </Suspense>
    ) : null;
  const table = <ParticipantTable tenant={tenant} slug={slug} />;
  if (tournament.mode !== "tdm" && !invites) return table;
  return (
    <Stack gap={7}>
      {tournament.mode === "tdm" && (
        <Suspense fallback={<SectionSkeleton tiles={2} />}>
          <TeamsSection tenant={tenant} slug={slug} />
        </Suspense>
      )}
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
      {invites}
    </Stack>
  );
}
