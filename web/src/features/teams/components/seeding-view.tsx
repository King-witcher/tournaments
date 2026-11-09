import { getTranslations } from "next-intl/server";

import { teamRating } from "@/features/ratings/lib/team-rating";
import { getRanking } from "@/features/ratings/server";
import { hasStarted, seedingOpen } from "@/features/tournaments/lib/status";
import { loadTournament } from "@/features/tournaments/server";
import { apiErrorMessage } from "@/shared/api/errors";
import type { components } from "@/shared/api/schema";
import { ButtonLink } from "@/shared/ui/button-link";
import { EmptyState } from "@/shared/ui/empty-state";
import { PageError } from "@/shared/ui/page-error";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import { getSeeding } from "../server";
import { SeedingEditor } from "./seeding-editor";
import { SeedingList } from "./seeding-list";
import { SeedingTeamsNotice } from "./seeding-teams-notice";

type Props = { tenant: string; slug: string };

/**
 * A seção Ordem da chave, para organizer e owner. Na revisão antes de iniciar,
 * o editor (`SeedingEditor`), com o que a ordem decide pelo formato da
 * primeira fase; no TDM, só os times completos entram, e o aviso dos
 * incompletos leva aos Times. Antes da revisão, o estado; depois de iniciar, a
 * mesma lista só para consulta, o registro do que definiu a chave. Para quem
 * joga, o aviso e o caminho de volta. Lê a sessão, então fica em `<Suspense>`.
 */
export async function SeedingView({ tenant, slug }: Props) {
  const [t, tNav, tTeam, loaded] = await Promise.all([
    getTranslations("Participants.seeding"),
    getTranslations("Nav"),
    getTranslations("Team"),
    loadTournament(tenant, slug),
  ]);
  if (loaded.kind !== "ok") return null;

  const { me, tournament } = loaded;
  const path = `/${tenant}/${slug}`;
  if (me.role === "player") {
    return (
      <PageError href={path} label={tNav("backToTournament")}>
        {await apiErrorMessage("tenant.forbidden")}
      </PageError>
    );
  }

  const hasCheckIn = tournament.check_in;
  const started = hasStarted(tournament.status);
  if (!started && !seedingOpen(tournament)) {
    return (
      <EmptyState>
        {t(hasCheckIn ? "summary.atCheckIn" : "summary.atRegistration")}
      </EmptyState>
    );
  }

  const [order, ranking] = await Promise.all([
    getSeeding(tenant, slug),
    started ? undefined : getRanking(tenant),
  ]);
  if (!order) {
    return (
      <PageError href={path} label={tNav("backToTournament")}>
        {await apiErrorMessage("internal")}
      </PageError>
    );
  }

  if (started) {
    return (
      <Section title={t("titleStarted", { count: order.length })}>
        <Panel>
          <SeedingList order={order} numbered />
        </Panel>
      </Section>
    );
  }

  // No TDM, a ordem é de times; os incompletos ficam fora.
  const teams = tournament.mode === "tdm";
  const notice = teams && (
    <SeedingTeamsNotice tenant={tenant} tournament={tournament} />
  );
  if (order.length === 0) {
    return (
      <>
        {notice}
        <EmptyState
          action={
            teams && (
              <ButtonLink href={`${path}/teams`} kind="tertiary">
                {tTeam("form")}
              </ButtonLink>
            )
          }
        >
          {teams
            ? t(hasCheckIn ? "emptyTeamsCheckIn" : "emptyTeams", {
                size: tournament.team_size,
              })
            : hasCheckIn
              ? t("empty")
              : t("emptyNoCheckIn")}
        </EmptyState>
      </>
    );
  }

  const players = new Map(
    ranking?.players.map((p) => [p.user_id, p.rating] as const) ?? [],
  );
  const ratings = new Map<number, number>();
  for (const team of order) {
    const rating = teamRating(team.members, players, ranking?.initial ?? 0);
    if (rating !== undefined) ratings.set(team.id, rating);
  }
  return (
    <>
      {notice}
      <SeedingEditor
        tenant={tenant}
        tournament={slug}
        title={
          teams
            ? t("titleTeams", { count: order.length })
            : t("title", {
                count: order.length,
                total: tournament.participants,
              })
        }
        description={t(firstStageKey(tournament))}
        initial={order}
        ratings={ratings}
      />
    </>
  );
}

/** A frase do que a ordem decide, pelo formato da primeira fase. */
function firstStageKey(tournament: components["schemas"]["Tournament"]) {
  const [stage] = tournament.stages;
  if (stage.format === "elimination") {
    return stage.bracket === "single" ? "whatSingle" : "whatDouble";
  }
  if (stage.format === "round_robin") {
    return stage.final_tie === "organizer"
      ? "whatRoundRobin"
      : "whatRoundRobinSeed";
  }
  return stage.format === "swiss" ? "whatSwiss" : "whatPlacement";
}
