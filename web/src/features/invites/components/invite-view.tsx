import { AspectRatio, Layer, Stack, Tile } from "@carbon/react";
import Image from "next/image";
import { getLocale, getTranslations } from "next-intl/server";

import { RequireLogin } from "@/features/auth/components/require-login";
import { apiErrorMessage } from "@/shared/api/errors";
import type { components } from "@/shared/api/schema";
import { Avatar } from "@/shared/ui/avatar";
import { PageError } from "@/shared/ui/page-error";
import { PageHeader } from "@/shared/ui/page-header";
import { getInvite, localizedName } from "../server";
import { AcceptInviteButton } from "./accept-invite-button";
import { InviteDetails } from "./invite-details";
import { InviteMembership } from "./invite-membership";
import styles from "./invite-view.module.scss";

type InvitedTournament = components["schemas"]["InvitedTournament"];

/**
 * Para qual comunidade (ou torneio, no link de inscrição) é o convite, com o
 * logo da comunidade no título e, no link de inscrição, a capa do torneio no
 * topo do `Tile`, com o que ele é e a ação de cada caso: aceitar; entrar ou criar
 * conta, voltando para cá; já é membro (ou inscrito); inscrição fechada.
 * Convite expirado ou inválido: o erro e o caminho de volta (de comunidade
 * em exclusão, com o texto do convite, e não o da escrita recusada). Fica em
 * `<Suspense>`.
 */
export async function InviteView({ token }: { token: string }) {
  const [t, tNav, locale, preview] = await Promise.all([
    getTranslations("Invite"),
    getTranslations("Nav"),
    getLocale(),
    getInvite(token),
  ]);

  const invite = preview?.data;
  if (!invite) {
    const code = preview?.error?.error ?? "internal";
    return (
      <>
        <PageHeader title={t("unavailable")} />
        <PageError href="/" label={tNav("backHome")}>
          {code === "tenant.deletion_scheduled"
            ? t("deletionScheduled")
            : await apiErrorMessage(code)}
        </PageError>
      </>
    );
  }
  const { tournament, viewer } = invite;
  const closed = tournament && (await closedReason(tournament));
  const member = viewer?.membership === "member";
  // Já é membro e não há o que aceitar (nem inscrição a fazer).
  const done = member && (!tournament || viewer.registered);
  // O botão já diz o que aceitar faz (entrar, inscrever-se ou os dois); a
  // nota só para quem volta, que recupera o apelido e o histórico.
  const notes = [
    viewer?.membership === "former" &&
      !done &&
      t("returning", {
        tenant: invite.tenant_name,
        nickname: viewer.nickname ?? "",
      }),
  ].filter((note) => typeof note === "string");

  return (
    <>
      <PageHeader
        title={
          tournament
            ? t("tournamentTitle", { name: localizedName(tournament, locale) })
            : t("title", { name: invite.tenant_name })
        }
        logo={
          invite.tenant_logo_url && (
            <Avatar shape="square" size="lg" src={invite.tenant_logo_url} />
          )
        }
      />
      <Tile>
        <Layer>
          <Stack gap={6}>
            {tournament?.cover_url && (
              <AspectRatio ratio="2x1" className={styles.cover}>
                <Image
                  src={tournament.cover_url}
                  alt=""
                  fill
                  unoptimized
                  className={styles.image}
                />
              </AspectRatio>
            )}
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

/** Por que o link de inscrição não aceita agora, se não aceita. */
async function closedReason(tournament: InvitedTournament) {
  // A inscrição fica aberta até o início, também durante o check-in.
  if (
    tournament.status !== "registration" &&
    tournament.status !== "check_in"
  ) {
    return apiErrorMessage("tournament.registration_closed");
  }
  if (tournament.participants >= tournament.max_participants) {
    return apiErrorMessage("tournament.full", {
      max: tournament.max_participants,
    });
  }
  return undefined;
}
