import { getLocale, getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { TournamentInviteSection } from "@/features/invites/components/tournament-invite-section";
import { BracketSection } from "@/features/matches/components/bracket-section";
import { CheckInPanel } from "@/features/participants/components/check-in-panel";
import { ParticipantSection } from "@/features/participants/components/participant-section";
import { RegistrationPanel } from "@/features/participants/components/registration-panel";
import { SeedingPanel } from "@/features/participants/components/seeding-panel";
import { Markdown } from "@/shared/ui/markdown";
import { localizedTexts } from "../lib/localize";
import { invitesAllowed } from "../lib/status";
import { loadTournament } from "../server";
import { ErrorBack } from "./error-back";
import { TournamentActions } from "./tournament-actions";
import { TournamentHeader } from "./tournament-header";
import { TournamentRules } from "./tournament-rules";
import { TournamentSetupSummary } from "./tournament-setup-summary";

type Props = { tenant: string; slug: string };

/**
 * Página do torneio, de cima para baixo: para organizer e owner, o quadro da
 * organização (estado, próximo passo, editar); a inscrição e o check-in de
 * quem vê; chave e classificação (depois de iniciar); descrição, formato,
 * seeding (organização), inscritos, links de inscrição (organização) e
 * regras. Textos no idioma de quem vê. Lê a sessão, então fica em
 * `<Suspense>`.
 */
export async function TournamentView({ tenant, slug }: Props) {
  const [t, tBracket, locale, loaded] = await Promise.all([
    getTranslations("Tournament"),
    getTranslations("Bracket"),
    getLocale(),
    loadTournament(tenant, slug),
  ]);
  if (loaded.kind === "login") {
    return <LoginPrompt message={t("loginRequired")} />;
  }
  if (loaded.kind === "error") {
    return <ErrorBack tenant={tenant} code={loaded.code} />;
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
        <TournamentActions tenant={tenant} tournament={tournament} now={now} />
      )}
      <RegistrationPanel tenant={tenant} tournament={tournament} />
      <CheckInPanel tenant={tenant} tournament={tournament} now={now} />
      <Suspense fallback={<p className="text-sm">{tBracket("loading")}</p>}>
        <BracketSection tenant={tenant} tournament={tournament} />
      </Suspense>
      {texts.description && <Markdown>{texts.description}</Markdown>}
      <TournamentSetupSummary tournament={tournament} />
      {organizer && <SeedingPanel tenant={tenant} tournament={tournament} />}
      <ParticipantSection
        tenant={tenant}
        tournament={tournament}
        organizer={organizer}
      />
      {organizer && invitesAllowed(tournament.status) && (
        <TournamentInviteSection
          tenant={tenant}
          tournament={tournament.slug}
          full={tournament.participants >= tournament.max_participants}
        />
      )}
      {texts.rules && <TournamentRules rules={texts.rules} />}
    </>
  );
}
