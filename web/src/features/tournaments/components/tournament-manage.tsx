import { Stack } from "@carbon/react";
import { Suspense } from "react";

import { LifecycleSection } from "@/features/lifecycle/components/lifecycle-section";
import { SeedingSummary } from "@/features/teams/components/seeding-summary";
import { TeamsSummary } from "@/features/teams/components/teams-summary";
import { apiErrorMessage } from "@/shared/api/errors";
import { Notice } from "@/shared/ui/notice";
import { SectionSkeleton } from "@/shared/ui/skeleton";
import { loadTournament } from "../server";

type Props = { tenant: string; slug: string };

/**
 * A aba Organização: o estado do torneio com o próximo passo (e o motivo
 * quando não dá), no TDM o resumo dos times (com "Formar times"), o resumo
 * do seeding. Os links de inscrição ficam na aba Inscritos. W.O. e
 * correções ficam na partida. Para
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
      {tournament.mode === "tdm" && (
        <Suspense fallback={<SectionSkeleton rows={1} />}>
          <TeamsSummary tenant={tenant} tournament={tournament} />
        </Suspense>
      )}
      <Suspense fallback={<SectionSkeleton rows={1} />}>
        <SeedingSummary tenant={tenant} tournament={tournament} />
      </Suspense>
    </Stack>
  );
}
