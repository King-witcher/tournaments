import { getLocale, getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { TournamentInviteSection } from "@/features/invites/components/tournament-invite-section";
import { LifecyclePanel } from "@/features/lifecycle/components/lifecycle-panel";
import { CheckInPanel } from "@/features/participants/components/check-in-panel";
import { ParticipantSection } from "@/features/participants/components/participant-section";
import { RegistrationPanel } from "@/features/participants/components/registration-panel";
import { SeedingPanel } from "@/features/participants/components/seeding-panel";
import { apiErrorMessage } from "@/shared/api/errors";
import { Alert } from "@/shared/ui/alert";
import { Markdown } from "@/shared/ui/markdown";
import { localizedTexts } from "../lib/localize";
import { invitesAllowed } from "../lib/status";
import { loadTournament } from "../server";
import { TournamentActions } from "./tournament-actions";
import { TournamentHeader } from "./tournament-header";
import { TournamentRules } from "./tournament-rules";
import { TournamentSetupSummary } from "./tournament-setup-summary";

type Props = { tenant: string; slug: string };

/**
 * Página do torneio: textos no idioma de quem vê, inscrição, check-in,
 * formato, inscritos e regras; para organizer e owner, as ações, o ciclo
 * (check-in, iniciar), o seeding e os links de inscrição. Lê a sessão, então
 * fica em `<Suspense>`.
 */
export async function TournamentView({ tenant, slug }: Props) {
  const [t, locale, loaded] = await Promise.all([
    getTranslations("Tournament"),
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
  const now = new Date();
  return (
    <>
      <TournamentHeader
        tenant={tenant}
        tenantName={me.name}
        name={texts.name}
        tournament={tournament}
      />
      {organizer && (
        <TournamentActions tenant={tenant} tournament={tournament} />
      )}
      {organizer && (
        <LifecyclePanel tenant={tenant} tournament={tournament} now={now} />
      )}
      <RegistrationPanel tenant={tenant} tournament={tournament} />
      <CheckInPanel tenant={tenant} tournament={tournament} now={now} />
      {texts.description && <Markdown>{texts.description}</Markdown>}
      <TournamentSetupSummary tournament={tournament} />
      {organizer && <SeedingPanel tenant={tenant} tournament={tournament} />}
      <ParticipantSection
        tenant={tenant}
        tournament={tournament}
        organizer={organizer}
      />
      {organizer && invitesAllowed(tournament.status) && (
        <TournamentInviteSection tenant={tenant} tournament={tournament.slug} />
      )}
      {texts.rules && <TournamentRules rules={texts.rules} />}
    </>
  );
}
