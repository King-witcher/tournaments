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

  return (
    <li className="flex items-center gap-3 py-2">
      <Avatar src={member.avatar_url} />
      <span className="flex flex-1 flex-col">
        <span className="font-medium">{member.nickname}</span>
        {member.display_name !== member.nickname && (
          <span className="text-xs opacity-60">{member.display_name}</span>
        )}
      </span>
      <span className="text-sm opacity-70">{tRoles(member.role)}</span>
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
    </li>
  );
}
