import { getLocale, getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { TournamentInviteSection } from "@/features/invites/components/tournament-invite-section";
import { ParticipantSection } from "@/features/participants/components/participant-section";
import { RegistrationPanel } from "@/features/participants/components/registration-panel";
import { Link } from "@/i18n/navigation";
import { apiErrorMessage } from "@/shared/api/errors";
import { Alert } from "@/shared/ui/alert";
import { Markdown } from "@/shared/ui/markdown";
import { localizedTexts } from "../lib/localize";
import { invitesAllowed, participantsEditable } from "../lib/status";
import { loadTournament } from "../server";
import { TournamentActions } from "./tournament-actions";
import { TournamentRules } from "./tournament-rules";
import { TournamentSetupSummary } from "./tournament-setup-summary";

type Props = { tenant: string; slug: string };

/**
 * Página do torneio: textos no idioma de quem vê, inscrição, formato,
 * inscritos e regras; para organizer e owner, as ações e os links de
 * inscrição. Lê a sessão, então fica em `<Suspense>`.
 */
export async function TournamentView({ tenant, slug }: Props) {
  const [t, tStatus, tVisibility, locale, loaded] = await Promise.all([
    getTranslations("Tournament"),
    getTranslations("TournamentStatus"),
    getTranslations("Visibility"),
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
  const organizer = me.role !== "player";
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
        <p className="text-sm opacity-70">
          {tStatus(tournament.status)} · {tVisibility(tournament.visibility)}
        </p>
      </header>
      {organizer && (
        <TournamentActions tenant={tenant} tournament={tournament} />
      )}
      <RegistrationPanel tenant={tenant} tournament={tournament} />
      {texts.description && <Markdown>{texts.description}</Markdown>}
      <TournamentSetupSummary tournament={tournament} />
      <ParticipantSection
        tenant={tenant}
        tournament={tournament.slug}
        max={tournament.max_participants}
        canManage={organizer && participantsEditable(tournament.status)}
      />
      {organizer && invitesAllowed(tournament.status) && (
        <TournamentInviteSection tenant={tenant} tournament={tournament.slug} />
      )}
      {texts.rules && <TournamentRules rules={texts.rules} />}
    </>
  );
}
