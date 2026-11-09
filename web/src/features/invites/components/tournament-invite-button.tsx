import { getTranslations } from "next-intl/server";

import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { CopyInviteLinkButton } from "./copy-invite-link-button";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
};

/**
 * "Copiar link de inscrição", ao lado do título da aba Inscritos (organização,
 * no rascunho e até o início). Com as vagas lotadas, o toast diz que o link
 * só volta a inscrever quando abrir vaga. Sem o token (a API falhou), nada: a
 * tabela continua.
 */
export async function TournamentInviteButton({ tenant, tournament }: Props) {
  const api = await serverApi();
  const [t, tInvite, token] = await Promise.all([
    getTranslations("Invite.tournamentLink"),
    getTranslations("Invite"),
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
    <CopyInviteLinkButton
      token={token}
      label={t("copy")}
      copied={full ? t("copiedFull") : tInvite("copied")}
      copiedKind={full ? "info" : "success"}
      kind="tertiary"
      size="sm"
    />
  );
}
