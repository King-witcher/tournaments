import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Avatar } from "@/shared/ui/avatar";
import { MemberRoleButton } from "./member-role-button";
import { RemoveMemberButton } from "./remove-member-button";

type Props = {
  slug: string;
  member: components["schemas"]["Member"];
  canChangeRoles: boolean;
  canRemove: boolean;
};

export function MemberRow({ slug, member, canChangeRoles, canRemove }: Props) {
  const tRoles = useTranslations("Roles");
  const changeRole = canChangeRoles && member.role !== "owner";

  return (
    <li className="flex flex-wrap items-center gap-x-3 gap-y-1 py-2">
      <Avatar src={member.avatar_url} name={member.nickname} />
      <span className="flex flex-1 flex-col">
        <span className="font-medium">{member.nickname}</span>
        {member.display_name !== member.nickname && (
          <span className="text-xs text-muted">{member.display_name}</span>
        )}
      </span>
      <span className="text-sm text-muted">{tRoles(member.role)}</span>
      {(changeRole || canRemove) && (
        // À direita; vão para a linha de baixo só quando não cabem (celular, várias ações).
        <span className="ml-auto flex flex-wrap justify-end gap-3">
          {canChangeRoles && member.role !== "owner" && (
            <MemberRoleButton
              slug={slug}
              userId={member.user_id}
              role={member.role}
            />
          )}
          {canRemove && (
            <RemoveMemberButton
              slug={slug}
              userId={member.user_id}
              nickname={member.nickname}
            />
          )}
        </span>
      )}
    </li>
  );
}
