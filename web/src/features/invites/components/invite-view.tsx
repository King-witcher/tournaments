import { getFormatter, getLocale, getTranslations } from "next-intl/server";

import { RequireLogin } from "@/features/auth/components/require-login";
import { apiErrorMessage } from "@/shared/api/errors";
import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { Alert } from "@/shared/ui/alert";
import { AcceptInviteButton } from "./accept-invite-button";

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
  const { tournament } = invite;
  const closed = tournament && (await closedReason(tournament));

  return (
    <>
      <h1 className="text-3xl font-semibold">
        {tournament
          ? t("tournamentTitle", { name: localizedName(tournament, locale) })
          : t("title", { name: invite.tenant_name })}
      </h1>
      {tournament && (
        <p className="text-sm opacity-70">
          {t("tournamentHint", { tenant: invite.tenant_name })}
        </p>
      )}
      {invite.expires_at && (
        <p className="text-sm opacity-70">
          {t("expires", {
            expires: format.relativeTime(
              new Date(invite.expires_at),
              new Date(),
            ),
          })}
        </p>
      )}
      {closed ? (
        <Alert>{closed}</Alert>
      ) : (
        <RequireLogin message={t("loginRequired")}>
          <AcceptInviteButton token={token} tournament={!!tournament} />
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
