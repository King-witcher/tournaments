import { getTranslations } from "next-intl/server";

import { serverApi } from "@/shared/api/server";
import { InviteLinkButton } from "./invite-link-button";

type Props = {
  slug: string;
  /** O nome da comunidade, no título do Modal. */
  name: string;
};

/**
 * "Convidar", na barra da tabela de membros (organizer e owner): o link único
 * da comunidade. Sem o token (a API falhou), nada: a tabela continua.
 */
export async function CommunityInviteButton({ slug, name }: Props) {
  const api = await serverApi();
  const [t, token] = await Promise.all([
    getTranslations("Invite.communityLink"),
    api
      .GET("/tenants/{slug}/invite", { params: { path: { slug } } })
      .then(({ data }) => data?.token)
      .catch(() => undefined),
  ]);
  if (!token) return null;

  return (
    <InviteLinkButton
      tenant={slug}
      token={token}
      label={t("open")}
      title={t("title", { name })}
      note={t("note")}
      linkLabel={t("link")}
      rotateConfirm={t("rotateConfirm")}
    />
  );
}
