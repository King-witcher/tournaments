import { useFormatter, useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { RevokeInviteButton } from "./revoke-invite-button";

type Props = {
  slug: string;
  invite: components["schemas"]["Invite"];
  /** Referência para "expira em ..."; a mesma para a lista inteira. */
  now: Date;
};

export function InviteRow({ slug, invite, now }: Props) {
  const t = useTranslations("Tenant.invites");
  const format = useFormatter();

  return (
    <li className="flex items-center gap-3 py-2">
      <span className="flex flex-1 flex-col text-sm">
        <span>
          {t("item", {
            name: invite.created_by,
            expires: format.relativeTime(new Date(invite.expires_at), now),
          })}
        </span>
        <span className="text-xs opacity-60">
          {t("uses", { count: invite.uses })}
        </span>
      </span>
      <RevokeInviteButton slug={slug} inviteId={invite.id} />
    </li>
  );
}
