import { getTranslations } from "next-intl/server";

import { serverApi } from "@/shared/api/server";
import { CopyInviteLinkButton } from "./copy-invite-link-button";

/**
 * "Copiar link de convite", na barra da tabela de membros (organizer e
 * owner). Sem o token (a API falhou), nada: a tabela continua.
 */
export async function CommunityInviteButton({ slug }: { slug: string }) {
  const api = await serverApi();
  const [t, tInvite, token] = await Promise.all([
    getTranslations("Invite.communityLink"),
    getTranslations("Invite"),
    api
      .GET("/tenants/{slug}/invite", { params: { path: { slug } } })
      .then(({ data }) => data?.token)
      .catch(() => undefined),
  ]);
  if (!token) return null;

  return (
    <CopyInviteLinkButton
      token={token}
      label={t("copy")}
      copied={tInvite("copied")}
    />
  );
}
