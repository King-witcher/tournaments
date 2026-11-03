import { getTranslations } from "next-intl/server";

import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { InviteLinkButton } from "./invite-link-button";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
};

/**
 * "Link de inscrição", ao lado do título da aba Inscritos (organização, no
 * rascunho e até o início): o link único do torneio. Sem o token (a API
 * falhou), nada: a tabela continua.
 */
export async function TournamentInviteButton({ tenant, tournament }: Props) {
  const api = await serverApi();
  const [t, token] = await Promise.all([
    getTranslations("Invite.tournamentLink"),
    api
      .GET("/tenants/{slug}/tournaments/{tournament}/invite", {
        params: { path: { slug: tenant, tournament: tournament.slug } },
      })
      .then(({ data }) => data?.token)
      .catch(() => undefined),
  ]);
  if (!token) return null;
  const full = tournament.participants >= tournament.max_participants;

  return (
    <InviteLinkButton
      tenant={tenant}
      tournament={tournament.slug}
      token={token}
      label={t("open")}
      title={t("open")}
      note={t("note")}
      linkLabel={t("open")}
      warning={full ? t("full") : undefined}
      rotateConfirm={t("rotateConfirm")}
      kind="tertiary"
      size="sm"
    />
  );
}
