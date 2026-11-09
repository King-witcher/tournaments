import { getLocale, getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { LiveMatches } from "@/features/matches/components/live-matches";
import { PendingMatches } from "@/features/matches/components/pending-matches";
import { loadBracket } from "@/features/matches/server";
import { MyParticipation } from "@/features/participants/components/my-participation";
import { NextStage } from "@/features/stages/components/next-stage";
import { pendingStage, withStageStatus } from "@/features/stages/lib/stages";
import { PageColumns } from "@/shared/ui/page-columns";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import { SectionSkeleton } from "@/shared/ui/skeleton";
import { UserText } from "@/shared/ui/user-text";
import { localizedTexts } from "../lib/localize";
import { hasStarted } from "../lib/status";
import { loadTournament } from "../server";
import { TournamentSetupSummary } from "./tournament-setup-summary";
import { TournamentStateSection } from "./tournament-state-section";

type Props = { tenant: string; slug: string };

/**
 * A Visão geral, o painel do torneio, em duas colunas a partir do `lg`. Na
 * principal, do que pede ação de quem vê ao que só informa: "Sua situação",
 * a fase seguinte à espera, as séries que esperam quem vê, as séries em
 * andamento, a descrição e o resumo do formato (com o link para as regras).
 * Na estreita, o estado do torneio: o ciclo, um passo embaixo do outro, com
 * o andamento das séries ou o campeão. O formato fica na principal porque os
 * valores dele são frases, que numa coluna estreita quebram em muitas linhas. O erro e o login já aparecem na casca; aqui, nada. Fica em
 * `<Suspense>`.
 */
export async function TournamentOverview({ tenant, slug }: Props) {
  const [t, locale, loaded] = await Promise.all([
    getTranslations("Tournament"),
    getLocale(),
    loadTournament(tenant, slug),
  ]);
  if (loaded.kind !== "ok") return null;

  const { me, tournament } = loaded;
  const organizer = me.role !== "player";
  const now = new Date();
  // A chave da fase atual: as pendências, o andamento e o campeão (a mesma
  // busca da seção Chave).
  const loadedBracket = hasStarted(tournament.status)
    ? await loadBracket(tenant, slug)
    : undefined;
  const bracket =
    loadedBracket?.kind === "ok"
      ? withStageStatus(tournament, loadedBracket.data)
      : undefined;
  const matchesPath = `/${tenant}/${slug}/matches`;
  const { description } = localizedTexts(tournament, locale);

  return (
    <PageColumns
      main={
        <>
          <MyParticipation
            tenant={tenant}
            tournament={tournament}
            organizer={organizer}
            now={now}
          />
          {pendingStage(tournament) && (
            <Suspense fallback={<SectionSkeleton rows={2} />}>
              <NextStage
                tenant={tenant}
                tournament={tournament}
                organizer={organizer}
              />
            </Suspense>
          )}
          {bracket && (
            <PendingMatches bracket={bracket} basePath={matchesPath} />
          )}
          {bracket?.status === "in_progress" && (
            <LiveMatches bracket={bracket} basePath={matchesPath} />
          )}
          {description && (
            <Panel>
              <Section title={t("description")}>
                <UserText>{description}</UserText>
              </Section>
            </Panel>
          )}
          <Panel>
            <TournamentSetupSummary
              tournament={tournament}
              rulesHref={`/${tenant}/${slug}/rules`}
            />
          </Panel>
        </>
      }
      aside={
        <Panel>
          <TournamentStateSection
            tournament={tournament}
            bracket={bracket}
            organizer={organizer}
            now={now}
          />
        </Panel>
      }
    />
  );
}
