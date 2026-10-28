import { getTranslations } from "next-intl/server";

import type { components } from "@/shared/api/schema";
import { ButtonLink } from "@/shared/ui/button-link";

type Props = {
  invite: components["schemas"]["InvitePreview"];
};

/**
 * Para quem já é membro (e, no link de inscrição, já está inscrito), no lugar
 * do botão de aceitar: diz que não há nada a fazer e leva à comunidade ou ao
 * torneio.
 */
export async function InviteMembership({ invite }: Props) {
  const { tournament } = invite;
  const t = await getTranslations("Invite");
  const href = tournament
    ? `/${invite.tenant_slug}/${tournament.slug}`
    : `/${invite.tenant_slug}`;
  return (
    <div className="flex flex-col items-start gap-3">
      <p>
        {tournament
          ? t("alreadyRegistered")
          : t("alreadyMember", { tenant: invite.tenant_name })}
      </p>
      <ButtonLink href={href}>
        {tournament ? t("goToTournament") : t("goToTenant")}
      </ButtonLink>
    </div>
  );
}
