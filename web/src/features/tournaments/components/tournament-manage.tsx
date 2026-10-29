import { Stack } from "@carbon/react";
import { Suspense } from "react";

import { TournamentInviteSection } from "@/features/invites/components/tournament-invite-section";
import { LifecycleSection } from "@/features/lifecycle/components/lifecycle-section";
import { SeedingSummary } from "@/features/teams/components/seeding-summary";
import { apiErrorMessage } from "@/shared/api/errors";
import { Notice } from "@/shared/ui/notice";
import { SectionSkeleton } from "@/shared/ui/skeleton";
import { invitesAllowed } from "../lib/status";
import { loadTournament } from "../server";

type Props = { tenant: string; slug: string };

/**
 * A aba Organização: o estado do torneio com o próximo passo (e o motivo
 * quando não dá), o resumo do seeding e, no rascunho e com a inscrição
 * aberta, os links de inscrição. W.O. e correções ficam na partida. Para
 * quem joga, o aviso de que a aba não é dele. Fica em `<Suspense>`.
 */
export async function TournamentManage({ tenant, slug }: Props) {
  const loaded = await loadTournament(tenant, slug);
  if (loaded.kind !== "ok") return null;
  if (loaded.me.role === "player") {
    return (
      <Notice kind="error">{await apiErrorMessage("tenant.forbidden")}</Notice>
    );
  }

  const { tournament } = loaded;
  return (
    <Stack gap={7}>
      <LifecycleSection
        tenant={tenant}
        tournament={tournament}
        now={new Date()}
      />
      <Suspense fallback={<SectionSkeleton rows={1} />}>
        <SeedingSummary tenant={tenant} tournament={tournament} />
      </Suspense>
      {invitesAllowed(tournament.status) && (
        <Suspense fallback={<SectionSkeleton rows={2} />}>
          <TournamentInviteSection tenant={tenant} tournament={tournament} />
        </Suspense>
      )}
    </Stack>
  );
}
