import { useFormatter, useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { RevokeInviteButton } from "./revoke-invite-button";

type Props = {
  tenant: string;
  invite: components["schemas"]["TournamentInvite"];
  /** Referência para "há ..."; a mesma para a lista inteira. */
  now: Date;
};

export function TournamentInviteRow({ tenant, invite, now }: Props) {
  const t = useTranslations("Tournament.invites");
  const format = useFormatter();

  return (
    <li className="flex items-center gap-3 py-2">
      <span className="flex flex-1 flex-col text-sm">
        <span>
          {t("item", {
            name: invite.created_by,
            created: format.relativeTime(new Date(invite.created_at), now),
          })}
        </span>
        <span className="text-xs opacity-60">
          {t("uses", { count: invite.uses })}
        </span>
      </span>
      <RevokeInviteButton slug={tenant} inviteId={invite.id} />
    </li>
  );
}
