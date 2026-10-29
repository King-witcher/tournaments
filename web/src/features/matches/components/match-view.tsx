import { Accordion } from "@carbon/react";
import { getLocale, getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { loadMatchPrints } from "@/features/prints/server";
import { ErrorBack } from "@/features/tournaments/components/error-back";
import { localizedTexts } from "@/features/tournaments/lib/localize";
import { loadTournament } from "@/features/tournaments/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { PageError } from "@/shared/ui/page-error";
import { loadBracket, loadMaps, loadMatch } from "../server";
import { CorrectionNote } from "./correction-note";
import { MatchGamesPanel } from "./match-games-panel";
import { MatchHeader } from "./match-header";
import { MatchHistory } from "./match-history";
import { MatchScoreboard } from "./match-scoreboard";
import { OrganizerPanel } from "./organizer-panel";
import { PendingMatches } from "./pending-matches";
import { VetoHistory } from "./veto-history";
import { VetoSection } from "./veto-section";

type Props = { tenant: string; slug: string; matchKey: string };

/**
 * Página da partida, de cima para baixo: o cabeçalho (trilha até a chave,
 * código e lados, estado), o placar da série, o veto (com o torneio que tem
 * veto: "Pronto", mapas, campeões), os mapas um por linha (enviar
 * o próximo, confirmar ou contestar; para a organização, confirmar na hora,
 * resolver, corrigir e anular), o aviso de correção recusada, as outras
 * partidas que esperam quem vê e, fechados no fim, o W.O. da organização e
 * o histórico (o da partida e o do veto). Lê a sessão: fica em `<Suspense>`.
 */
export async function MatchView({ tenant, slug, matchKey }: Props) {
  // Os prints de cada mapa (GamePrints) saem deste pedido: junto dos outros.
  void loadMatchPrints(tenant, slug, matchKey);
  const [t, tNav, locale, loaded, found, bracket] = await Promise.all([
    getTranslations("Match"),
    getTranslations("Nav"),
    getLocale(),
    loadTournament(tenant, slug),
    loadMatch(tenant, slug, matchKey),
    loadBracket(tenant, slug),
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
  const names = { a: detail.a?.nickname ?? "", b: detail.b?.nickname ?? "" };
  const playable = s.state === "ready" || s.state === "completed";
  const needsMaps = detail.next_game != null || detail.can_manage;
  const maps = needsMaps ? await loadMaps() : [];
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
      <VetoSection
        target={target}
        detail={detail}
        rule={tournament.champion_rules.no_repeat?.scope ?? null}
      />
      <MatchGamesPanel
        target={target}
        detail={detail}
        maps={maps}
        contestMinutes={tournament.contest_minutes}
      />
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
      <Accordion align="start">
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
