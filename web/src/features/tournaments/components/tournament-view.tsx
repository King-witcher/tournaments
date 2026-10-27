import { getLocale, getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { Link } from "@/i18n/navigation";
import { apiErrorMessage } from "@/shared/api/errors";
import { Alert } from "@/shared/ui/alert";
import { Markdown } from "@/shared/ui/markdown";
import { localizedTexts } from "../lib/localize";
import { loadTournament } from "../server";
import { TournamentActions } from "./tournament-actions";
import { TournamentRules } from "./tournament-rules";
import { TournamentSetupSummary } from "./tournament-setup-summary";

type Props = { tenant: string; slug: string };

/**
 * Página do torneio: textos no idioma de quem vê, formato e regras; para
 * organizer e owner, as ações. Lê a sessão, então fica em `<Suspense>`.
 */
export async function TournamentView({ tenant, slug }: Props) {
  const [t, tStatus, locale, loaded] = await Promise.all([
    getTranslations("Tournament"),
    getTranslations("TournamentStatus"),
    getLocale(),
    loadTournament(tenant, slug),
  ]);
  if (loaded.kind === "login") {
    return <LoginPrompt message={t("loginRequired")} />;
  }
  if (loaded.kind === "error") {
    return <Alert>{await apiErrorMessage(loaded.code)}</Alert>;
  }

  const { me, tournament } = loaded;
  const texts = localizedTexts(tournament, locale);
  return (
    <>
      <header className="flex flex-col gap-1">
        <Link
          href={`/${tenant}`}
          className="text-sm opacity-70 hover:underline"
        >
          {me.name}
        </Link>
        <h1 className="text-3xl font-semibold">{texts.name}</h1>
        <p className="text-sm opacity-70">{tStatus(tournament.status)}</p>
      </header>
      {me.role !== "player" && (
        <TournamentActions tenant={tenant} tournament={tournament} />
      )}
      {texts.description && <Markdown>{texts.description}</Markdown>}
      <TournamentSetupSummary tournament={tournament} />
      {texts.rules && <TournamentRules rules={texts.rules} />}
    </>
  );
}
