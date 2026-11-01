import { Accordion } from "@carbon/react";
import { getLocale, getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { getCurrentUser } from "@/features/auth/server";
import { LiveUpdates } from "@/features/realtime/components/live-updates";
import { loadMatchScreenshots } from "@/features/screenshots/server";
import { getTeamName } from "@/features/teams/server";
import { ErrorBack } from "@/features/tournaments/components/error-back";
import { localizedTexts } from "@/features/tournaments/lib/localize";
import { loadTournament } from "@/features/tournaments/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { PageError } from "@/shared/ui/page-error";
import { loadBracket, loadMaps, loadMatch } from "../server";
import { CorrectionNote } from "./correction-note";
import { LiveMatchProvider } from "./live-match-provider";
import { MatchGamesPanel } from "./match-games-panel";
import { MatchHeader } from "./match-header";
import { MatchHistory } from "./match-history";
import { MatchScoreboard } from "./match-scoreboard";
import { MatchSteps } from "./match-steps";
import { OrganizerPanel } from "./organizer-panel";
import { PendingMatches } from "./pending-matches";
import { VetoHistory } from "./veto-history";

type Props = { tenant: string; slug: string; matchKey: string };

/**
 * Página da série, de cima para baixo: o cabeçalho (trilha até a chave,
 * código e lados, estado), o placar da série, os passos (com o torneio que
 * tem veto: "Pronto", mapas, campeões, partidas; sem, só as partidas: as
 * enviadas numa linha cada, com confirmar, contestar e, para a organização,
 * confirmar na hora, resolver, corrigir e anular, e a da vez num cartão com
 * o envio), o aviso de correção recusada, as outras séries que esperam quem
 * vê e, fechados no fim, o W.O. da organização e o histórico (o da série e o
 * do veto). Lê a sessão: fica em `<Suspense>`.
 */
export async function MatchView({ tenant, slug, matchKey }: Props) {
  // Os screenshots de cada mapa (GameScreenshots) saem deste pedido: junto dos
  // outros.
  void loadMatchScreenshots(tenant, slug, matchKey);
  const [t, tNav, locale, loaded, found, bracket, maps, user, teamName] =
    await Promise.all([
      getTranslations("Match"),
      getTranslations("Nav"),
      getLocale(),
      loadTournament(tenant, slug),
      loadMatch(tenant, slug, matchKey),
      loadBracket(tenant, slug),
      loadMaps(),
      getCurrentUser(),
      getTeamName(),
    ]);
  if (loaded.kind === "login") {
    return <LoginPrompt message={t("loginRequired")} />;
  }
  if (loaded.kind === "error") {
    return <ErrorBack tenant={tenant} code={loaded.code} />;
  }
  if (found.kind === "error") {
    return (
      <PageError href={`/${tenant}/${slug}`} label={tNav("backToTournament")}>
        {await apiErrorMessage(found.code)}
      </PageError>
    );
  }

  const detail = found.data;
  const tournament = loaded.tournament;
  const s = detail.summary;
  const target = { tenant, slug, matchKey: s.key };
  const names = {
    a: detail.a ? teamName(detail.a) : "",
    b: detail.b ? teamName(detail.b) : "",
  };
  const playable = s.state === "ready" || s.state === "completed";
  const r = s.result;
  const basePath = `/${tenant}/${slug}/matches`;
  return (
    <>
      <MatchHeader
        tenant={tenant}
        slug={slug}
        tenantName={loaded.me.name}
        tournamentName={localizedTexts(tournament, locale).name}
        detail={detail}
      />
      <MatchScoreboard tenant={tenant} detail={detail} />
      {s.state === "ready" && detail.veto && detail.veto.status !== "done" && (
        // O veto anda ao vivo (cada passo tem 30 s): recarrega mais rápido.
        <LiveUpdates tenant={tenant} tournament={slug} match={s.key} fast />
      )}
      <LiveMatchProvider
        initial={detail}
        viewerId={user?.id ?? null}
        bansPerSeries={tournament.champion_rules.bans_per_series}
        vetoOrder={tournament.veto.order}
      >
        <MatchSteps
          target={target}
          rule={tournament.champion_rules.no_repeat ?? null}
          games={
            <MatchGamesPanel
              target={target}
              detail={detail}
              maps={maps}
              titled={!detail.veto}
            />
          }
        />
      </LiveMatchProvider>
      {detail.can_manage &&
        s.state === "completed" &&
        bracket.kind === "ok" && (
          <CorrectionNote
            bracket={bracket.data}
            matchKey={s.key}
            basePath={basePath}
          />
        )}
      {bracket.kind === "ok" && (
        // O que mais espera quem vê, para seguir depois de agir nesta.
        <PendingMatches
          bracket={bracket.data}
          basePath={basePath}
          exclude={matchKey}
        />
      )}
      <Accordion>
        {detail.can_manage && playable && (
          // A chave muda a cada resultado: o bloco fecha depois de salvar.
          <OrganizerPanel
            key={`${r?.winner}:${r?.walkover}:${r?.a_wins}:${r?.b_wins}`}
            target={target}
            names={names}
          />
        )}
        <MatchHistory detail={detail} />
        {detail.veto && <VetoHistory veto={detail.veto} names={names} />}
      </Accordion>
    </>
  );
}
