import { getFormatter, getLocale, getTranslations } from "next-intl/server";

import { RequireLogin } from "@/features/auth/components/require-login";
import { apiErrorMessage } from "@/shared/api/errors";
import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { Alert } from "@/shared/ui/alert";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { AcceptInviteButton } from "./accept-invite-button";
import { InviteMembership } from "./invite-membership";

type InvitedTournament = components["schemas"]["InvitedTournament"];

/**
 * Para qual comunidade (ou torneio, no link de inscrição) é o convite; aceitar
 * exige login. Fica em `<Suspense>`.
 */
export async function InviteView({ token }: { token: string }) {
  const api = await serverApi();
  const [t, format, locale, preview] = await Promise.all([
    getTranslations("Invite"),
    getFormatter(),
    getLocale(),
    api
      .GET("/invites/{token}", { params: { path: { token } } })
      .catch(() => undefined),
  ]);

  const invite = preview?.data;
  if (!invite) {
    return (
      <Alert>
        {await apiErrorMessage(preview?.error?.error ?? "internal")}
      </Alert>
    );
  }
  const { tournament, viewer } = invite;
  const closed = tournament && (await closedReason(tournament));
  const member = viewer?.membership === "member";
  // Já é membro e não há o que aceitar (nem inscrição a fazer).
  const done = member && (!tournament || viewer.registered);

  return (
    <>
      <h1 className="text-3xl font-semibold">
        {tournament
          ? t("tournamentTitle", { name: localizedName(tournament, locale) })
          : t("title", { name: invite.tenant_name })}
      </h1>
      {tournament && (
        <p className="text-sm text-muted">
          {t("tournamentSpots", {
            count: tournament.participants,
            max: tournament.max_participants,
          })}{" "}
          {!member && t("tournamentHint", { tenant: invite.tenant_name })}
        </p>
      )}
      {tournament?.starts_at && (
        <p className="text-sm text-muted">
          {t("startsAt")} <LocalDateTime value={tournament.starts_at} />
        </p>
      )}
      {invite.expires_at && (
        <p className="text-sm text-muted">
          {t("expires", {
            expires: format.relativeTime(
              new Date(invite.expires_at),
              new Date(),
            ),
          })}
        </p>
      )}
      {viewer?.membership === "former" && !done && (
        <p>
          {t("returning", {
            tenant: invite.tenant_name,
            nickname: viewer.nickname ?? "",
          })}
        </p>
      )}
      {member && tournament && !done && !closed && (
        <p>{t("memberRegisters", { tenant: invite.tenant_name })}</p>
      )}
      {done ? (
        <InviteMembership invite={invite} />
      ) : closed ? (
        <Alert>{closed}</Alert>
      ) : (
        <RequireLogin message={t("loginRequired")}>
          <AcceptInviteButton
            token={token}
            tournament={!!tournament}
            member={member}
          />
        </RequireLogin>
      )}
    </>
  );
}

/**
 * Nome no idioma de quem vê, como o `localizedName` de `tournaments` (que
 * não é importado: aquela feature já monta os links de inscrição desta).
 */
function localizedName(tournament: InvitedTournament, locale: string) {
  return (
    tournament.translations.find((tr) => tr.locale === locale)?.name ??
    tournament.name
  );
}

/** Por que o link de inscrição não aceita agora, se não aceita. */
async function closedReason(tournament: InvitedTournament) {
  if (tournament.status !== "registration") {
    return apiErrorMessage("tournament.registration_closed");
  }
  if (tournament.participants >= tournament.max_participants) {
    return apiErrorMessage("tournament.full", {
      max: tournament.max_participants,
    });
  }
  return undefined;
}
