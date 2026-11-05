import { getLocale, getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { getCurrentUser } from "@/features/auth/server";
import { loadMatchScreenshots } from "@/features/screenshots/server";
import { getTeamName } from "@/features/teams/server";
import { ErrorBack } from "@/features/tournaments/components/error-back";
import { localizedTexts } from "@/features/tournaments/lib/localize";
import { loadTournament } from "@/features/tournaments/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { preload } from "@/shared/lib/preload";
import { PageError } from "@/shared/ui/page-error";
import { loadBracket, loadMaps, loadMatch } from "../server";
import { MatchAfter } from "./match-after";
import { MatchHeader } from "./match-header";
import { MatchLive } from "./match-live";

type Props = { tenant: string; slug: string; matchKey: string };

/**
 * Página da série, de cima para baixo: o cabeçalho (trilha até a chave,
 * código e lados com o placar depois da primeira partida enviada, estado e,
 * embaixo, como terminou), os passos (Pronto, Mapas e Campeões com veto no
 * torneio, e uma partida por passo, cada uma num cartão com o envio ou o
 * resultado e o que dá para fazer nela), o aviso de correção recusada, as
 * outras séries que esperam quem vê e, fechados no fim, o W.O. da
 * organização e o histórico (o da série e o do veto). Lê a sessão: fica em
 * `<Suspense>`.
 */
export async function MatchView({ tenant, slug, matchKey }: Props) {
  // Os screenshots de cada partida (GameScreenshots) saem deste pedido:
  // junto dos outros.
  preload(loadMatchScreenshots(tenant, slug, matchKey));
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
  return (
    <>
      <MatchHeader
        tenant={tenant}
        slug={slug}
        tenantName={loaded.me.name}
        tournamentName={localizedTexts(tournament, locale).name}
        detail={detail}
      />
      <MatchLive
        target={target}
        detail={detail}
        tournament={tournament}
        maps={maps}
        viewerId={user?.id ?? null}
      />
      <MatchAfter
        target={target}
        detail={detail}
        bracket={bracket.kind === "ok" ? bracket.data : null}
        names={names}
      />
    </>
  );
}
