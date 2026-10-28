import { Layer, Stack, Tile } from "@carbon/react";
import { getLocale, getTranslations } from "next-intl/server";

import { RequireLogin } from "@/features/auth/components/require-login";
import { apiErrorMessage } from "@/shared/api/errors";
import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { PageError } from "@/shared/ui/page-error";
import { PageHeader } from "@/shared/ui/page-header";
import { AcceptInviteButton } from "./accept-invite-button";
import { InviteDetails } from "./invite-details";
import { InviteMembership } from "./invite-membership";
import styles from "./invite-view.module.scss";

type InvitedTournament = components["schemas"]["InvitedTournament"];

/**
 * Para qual comunidade (ou torneio, no link de inscrição) é o convite, num
 * `Tile` com o que ele é e a ação de cada caso: aceitar; entrar ou criar
 * conta, voltando para cá; já é membro (ou inscrito); inscrição fechada.
 * Convite expirado ou inválido: o erro e o caminho de volta. Fica em
 * `<Suspense>`.
 */
export async function InviteView({ token }: { token: string }) {
  const api = await serverApi();
  const [t, tNav, locale, preview] = await Promise.all([
    getTranslations("Invite"),
    getTranslations("Nav"),
    getLocale(),
    api
      .GET("/invites/{token}", { params: { path: { token } } })
      .catch(() => undefined),
  ]);

  const invite = preview?.data;
  if (!invite) {
    return (
      <>
        <PageHeader title={t("unavailable")} />
        <PageError href="/" label={tNav("backHome")}>
          {await apiErrorMessage(preview?.error?.error ?? "internal")}
        </PageError>
      </>
    );
  }
  const { tournament, viewer } = invite;
  const closed = tournament && (await closedReason(tournament));
  const member = viewer?.membership === "member";
  // Já é membro e não há o que aceitar (nem inscrição a fazer).
  const done = member && (!tournament || viewer.registered);
  const notes = [
    tournament &&
      !member &&
      !closed &&
      t("tournamentHint", { tenant: invite.tenant_name }),
    viewer?.membership === "former" &&
      !done &&
      t("returning", {
        tenant: invite.tenant_name,
        nickname: viewer.nickname ?? "",
      }),
    member &&
      tournament &&
      !done &&
      !closed &&
      t("memberRegisters", { tenant: invite.tenant_name }),
  ].filter((note) => typeof note === "string");

  return (
    <>
      <PageHeader
        title={
          tournament
            ? t("tournamentTitle", { name: localizedName(tournament, locale) })
            : t("title", { name: invite.tenant_name })
        }
      />
      <Tile>
        <Layer>
          <Stack gap={6}>
            <InviteDetails invite={invite} />
            {notes.map((note) => (
              <p key={note} className={styles.note}>
                {note}
              </p>
            ))}
            {done ? (
              <InviteMembership invite={invite} />
            ) : closed ? (
              <PageError
                href={
                  member ? `/${invite.tenant_slug}/${tournament.slug}` : "/"
                }
                label={member ? t("goToTournament") : tNav("backHome")}
              >
                {closed}
              </PageError>
            ) : (
              <RequireLogin message={t("loginRequired")}>
                <AcceptInviteButton
                  token={token}
                  tournament={!!tournament}
                  member={member}
                />
              </RequireLogin>
            )}
          </Stack>
        </Layer>
      </Tile>
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
