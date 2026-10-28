import { getTranslations } from "next-intl/server";

import type { components } from "@/shared/api/schema";
import { getMembers } from "../server";
import { MemberRow } from "./member-row";

type Role = components["schemas"]["Role"];

type Props = {
  slug: string;
  /** Papel de quem está vendo: o owner troca papéis e remove; o organizer remove players. */
  viewerRole: Role;
};

/** Quem `viewer` pode remover (a API confere de novo). */
function canRemove(viewer: Role, member: Role) {
  if (member === "owner") return false;
  return viewer === "owner" || (viewer === "organizer" && member === "player");
}

/** Membros do tenant. Fica em `<Suspense>`. */
export async function MemberList({ slug, viewerRole }: Props) {
  const [t, members] = await Promise.all([
    getTranslations("Tenant.members"),
    getMembers(slug),
  ]);

  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-xl font-semibold">
        {t("title", { count: members.length })}
      </h2>
      <ul className="divide-y divide-black/10 dark:divide-white/10">
        {members.map((member) => (
          <MemberRow
            key={member.user_id}
            slug={slug}
            member={member}
            canChangeRoles={viewerRole === "owner"}
            canRemove={canRemove(viewerRole, member.role)}
          />
        ))}
      </ul>
    </section>
  );
}
