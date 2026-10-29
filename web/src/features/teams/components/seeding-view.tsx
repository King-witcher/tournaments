import { Stack } from "@carbon/react";
import { getLocale, getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { teamRating } from "@/features/ratings/lib/team-rating";
import { getRanking } from "@/features/ratings/server";
import { ErrorBack } from "@/features/tournaments/components/error-back";
import { localizedName } from "@/features/tournaments/lib/localize";
import { seedingOpen } from "@/features/tournaments/lib/status";
import { loadTournament } from "@/features/tournaments/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { ButtonLink } from "@/shared/ui/button-link";
import { EmptyState } from "@/shared/ui/empty-state";
import { PageError } from "@/shared/ui/page-error";
import { PageHeader } from "@/shared/ui/page-header";
import { getSeeding } from "../server";
import { SeedingEditor } from "./seeding-editor";
import { SeedingTeamsNotice } from "./seeding-teams-notice";
import styles from "./seeding-view.module.scss";

type Props = { tenant: string; slug: string };

/**
 * A página do seeding, para organizer e owner, na revisão antes de iniciar:
 * quantos entram na chave, a explicação e a lista de times com a posição (e o
 * rating: a média dos jogadores; no duelo, o do jogador) de cada um,
 * "Sortear", "Ordenar por rating" e "Salvar". No TDM, só os times completos
 * entram, e o aviso dos incompletos leva à formação. Fora da revisão (ou para
 * quem joga), o aviso e o caminho de volta. Lê a sessão, então fica em
 * `<Suspense>`.
 */
export async function SeedingView({ tenant, slug }: Props) {
  const [t, tNav, tTournament, tTeam, locale, loaded] = await Promise.all([
    getTranslations("Participants.seeding"),
    getTranslations("Nav"),
    getTranslations("Tournament"),
    getTranslations("Team"),
    getLocale(),
    loadTournament(tenant, slug),
  ]);
  if (loaded.kind === "login") {
    return <LoginPrompt message={tTournament("loginRequired")} />;
  }
  if (loaded.kind === "error") {
    return <ErrorBack tenant={tenant} code={loaded.code} />;
  }

  const { me, tournament } = loaded;
  const path = `/${tenant}/${slug}`;
  const manage = `${path}/manage`;
  const breadcrumbs = [
    { href: "/", label: tNav("home") },
    { href: `/${tenant}`, label: me.name },
    { href: path, label: localizedName(tournament, locale) },
  ];

  if (me.role === "player") {
    return (
      <>
        <PageHeader breadcrumbs={breadcrumbs} title={t("pageTitle")} />
        <PageError href={path} label={tNav("backToTournament")}>
          {await apiErrorMessage("tenant.forbidden")}
        </PageError>
      </>
    );
  }
  const [order, ranking] = seedingOpen(tournament)
    ? await Promise.all([getSeeding(tenant, slug), getRanking(tenant)])
    : [undefined, undefined];
  if (!order) {
    // Fora da revisão (rascunho, inscrição com check-in, depois de iniciar)
    // ou a API falhou.
    return (
      <>
        <PageHeader breadcrumbs={breadcrumbs} title={t("pageTitle")} />
        <PageError href={manage} label={t("backToManage")}>
          {seedingOpen(tournament)
            ? await apiErrorMessage("internal")
            : t("closed")}
        </PageError>
      </>
    );
  }

  const hasCheckIn = tournament.check_in_minutes != null;
  const players = new Map(
    ranking?.players.map((p) => [p.user_id, p.rating] as const) ?? [],
  );
  const ratings = new Map<number, number>();
  for (const team of order) {
    const rating = teamRating(team.members, players, ranking?.initial ?? 0);
    if (rating !== undefined) ratings.set(team.id, rating);
  }
  // No TDM, a ordem é de times; os incompletos ficam fora e a formação, em
  // outra página.
  const teams = tournament.mode === "tdm";
  const size = tournament.team_size;
  return (
    <>
      <PageHeader
        breadcrumbs={breadcrumbs}
        title={
          teams
            ? t("titleTeams", { count: order.length })
            : t("title", {
                count: order.length,
                total: tournament.participants,
              })
        }
      >
        <Stack gap={3} className={styles.hint}>
          <p>{t("what")}</p>
          <p>{t("hint")}</p>
        </Stack>
      </PageHeader>
      {teams && <SeedingTeamsNotice tenant={tenant} tournament={tournament} />}
      {order.length === 0 ? (
        <EmptyState
          action={
            teams ? (
              <ButtonLink href={`${path}/teams`} kind="tertiary">
                {tTeam("form")}
              </ButtonLink>
            ) : (
              <ButtonLink href={manage} kind="tertiary">
                {t("backToManage")}
              </ButtonLink>
            )
          }
        >
          {teams
            ? t(hasCheckIn ? "emptyTeamsCheckIn" : "emptyTeams", { size })
            : hasCheckIn
              ? t("empty")
              : t("emptyNoCheckIn")}
        </EmptyState>
      ) : (
        <SeedingEditor
          // Recomeça do que a API devolveu a cada atualização da página.
          key={order.map((team) => `${team.id}:${team.seed}`).join(",")}
          tenant={tenant}
          tournament={slug}
          initial={order}
          ratings={ratings}
        />
      )}
    </>
  );
}
