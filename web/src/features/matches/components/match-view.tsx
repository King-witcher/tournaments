import { getLocale, getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
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

type Props = { tenant: string; slug: string; matchKey: string };

/**
 * Página da partida: o placar da série, os mapas um por linha (enviar o
 * próximo, confirmar ou contestar; para a organização, confirmar na hora,
 * resolver e corrigir), o W.O. da organização e o histórico. Lê a sessão:
 * fica em `<Suspense>`.
 */
export async function MatchView({ tenant, slug, matchKey }: Props) {
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
  return (
    <>
      <MatchHeader
        tenant={tenant}
        slug={slug}
        tenantName={loaded.me.name}
        tournamentName={localizedTexts(tournament, locale).name}
        detail={detail}
      />
      <MatchScoreboard detail={detail} />
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
            basePath={`/${tenant}/${slug}/matches`}
          />
        )}
      {detail.can_manage && playable && (
        <OrganizerPanel
          target={target}
          names={names}
          resultKey={`${r?.winner}:${r?.walkover}:${r?.a_wins}:${r?.b_wins}`}
        />
      )}
      {bracket.kind === "ok" && (
        // O que mais espera quem vê, para seguir depois de agir nesta.
        <PendingMatches
          bracket={bracket.data}
          basePath={`/${tenant}/${slug}/matches`}
          exclude={matchKey}
        />
      )}
      <MatchHistory detail={detail} />
    </>
  );
}
