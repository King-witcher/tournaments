import { getLocale, getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { ErrorBack } from "@/features/tournaments/components/error-back";
import { localizedTexts } from "@/features/tournaments/lib/localize";
import { loadTournament } from "@/features/tournaments/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { PageError } from "@/shared/ui/page-error";
import { loadMaps, loadMatch } from "../server";
import { MatchActions } from "./match-actions";
import { MatchGames } from "./match-games";
import { MatchHeader } from "./match-header";
import { MatchHistory } from "./match-history";
import { MatchScoreboard } from "./match-scoreboard";

type Props = { tenant: string; slug: string; matchKey: string };

/**
 * Página da partida: lados e placar, o que quem vê pode fazer (enviar,
 * confirmar ou contestar; para a organização, resolver, lançar, corrigir e
 * dar W.O.), detalhe por mapa e histórico. Lê a sessão: fica em `<Suspense>`.
 */
export async function MatchView({ tenant, slug, matchKey }: Props) {
  const [t, tNav, locale, loaded, found] = await Promise.all([
    getTranslations("Match"),
    getTranslations("Nav"),
    getLocale(),
    loadTournament(tenant, slug),
    loadMatch(tenant, slug, matchKey),
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
  const needsMaps = detail.summary.task === "report" || detail.can_manage;
  const maps = needsMaps ? await loadMaps() : [];
  return (
    <>
      <MatchHeader
        tenant={tenant}
        slug={slug}
        tournamentName={localizedTexts(tournament, locale).name}
        detail={detail}
      />
      <MatchScoreboard detail={detail} />
      <MatchActions
        tenant={tenant}
        slug={slug}
        detail={detail}
        maps={maps}
        contestMinutes={tournament.contest_minutes}
      />
      {detail.games.length > 0 && <MatchGames detail={detail} />}
      <MatchHistory detail={detail} />
    </>
  );
}
