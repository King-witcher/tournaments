import { Suspense } from "react";
import { LifecycleSection } from "@/features/lifecycle/components/lifecycle-section";
import { LiveMatches } from "@/features/matches/components/live-matches";
import { PendingMatches } from "@/features/matches/components/pending-matches";
import { loadBracket } from "@/features/matches/server";
import { NextStage } from "@/features/stages/components/next-stage";
import { pendingStage, withStageStatus } from "@/features/stages/lib/stages";
import { SeedingSummary } from "@/features/teams/components/seeding-summary";
import { TeamsSummary } from "@/features/teams/components/teams-summary";
import { apiErrorMessage } from "@/shared/api/errors";
import type { components } from "@/shared/api/schema";
import { Notice } from "@/shared/ui/notice";
import { PageColumns } from "@/shared/ui/page-columns";
import { Panel } from "@/shared/ui/panel";
import { SectionSkeleton } from "@/shared/ui/skeleton";
import { hasStarted } from "../lib/status";
import { loadTournament } from "../server";

type Props = { tenant: string; slug: string };

/** O que espera a organização numa série: a contestação a resolver. */
const ORGANIZER_TASKS: components["schemas"]["ViewerTask"][] = ["resolve"];

/**
 * A Organização, onde a organização acompanha o torneio, em duas colunas a
 * partir do `lg`. Na principal, o que pede ação: o estado com o próximo
 * passo (e o motivo quando não dá), a fase seguinte à espera, as séries que
 * esperam a organização (contestações), as séries em andamento e, no TDM, o
 * resumo dos times (com "Formar times"). Na estreita, o resumo da ordem da
 * chave. Os links de inscrição ficam em Inscritos; W.O. e correções, na
 * partida. Para quem joga, o aviso de que a seção não é dele. Fica em
 * `<Suspense>`.
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
  const loadedBracket = hasStarted(tournament.status)
    ? await loadBracket(tenant, slug)
    : undefined;
  const bracket =
    loadedBracket?.kind === "ok"
      ? withStageStatus(tournament, loadedBracket.data)
      : undefined;
  const matchesPath = `/${tenant}/${slug}/matches`;
  return (
    <PageColumns
      main={
        <>
          <LifecycleSection
            tenant={tenant}
            tournament={tournament}
            now={new Date()}
            bracket={bracket}
          />
          {pendingStage(tournament) && (
            <Suspense fallback={<SectionSkeleton rows={2} />}>
              <NextStage tenant={tenant} tournament={tournament} organizer />
            </Suspense>
          )}
          {bracket && (
            <PendingMatches
              bracket={bracket}
              basePath={matchesPath}
              tasks={ORGANIZER_TASKS}
            />
          )}
          {bracket?.status === "in_progress" && (
            <LiveMatches bracket={bracket} basePath={matchesPath} />
          )}
          {tournament.mode === "tdm" && (
            <Panel>
              <Suspense fallback={<SectionSkeleton rows={1} />}>
                <TeamsSummary tenant={tenant} tournament={tournament} />
              </Suspense>
            </Panel>
          )}
        </>
      }
      aside={
        <Panel>
          <Suspense fallback={<SectionSkeleton rows={1} />}>
            <SeedingSummary tenant={tenant} tournament={tournament} />
          </Suspense>
        </Panel>
      }
    />
  );
}
