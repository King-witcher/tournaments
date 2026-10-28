import { getTranslations } from "next-intl/server";

import { getTenant } from "@/features/tenants/server";
import type { components } from "@/shared/api/schema";
import { DataTable } from "@/shared/ui/data-table";
import { StatusTag } from "@/shared/ui/status-tag";
import { getMembers } from "../server";
import { MemberActions } from "./member-actions";
import { MemberName } from "./member-name";
import { MemberSince } from "./member-since";

type Role = components["schemas"]["Role"];

/** A tabela, que recebe o foco quando um membro removido some dela. */
const MEMBERS_ID = "members";

/** Quem `viewer` pode remover (a API confere de novo). */
function canRemove(viewer: Role, member: Role) {
  if (member === "owner") return false;
  return viewer === "owner" || (viewer === "organizer" && member === "player");
}

/**
 * A aba Membros: a tabela (membro com avatar, papel, desde), com busca; a
 * linha abre a página da pessoa. O owner troca papéis e remove; o
 * organizador remove jogadores. Fica em `<Suspense>`.
 */
export async function MemberList({ slug }: { slug: string }) {
  const [loaded, t, tRoles, members] = await Promise.all([
    getTenant(slug),
    getTranslations("Tenant.members"),
    getTranslations("Roles"),
    getMembers(slug),
  ]);
  // Sem acesso, o cabeçalho da comunidade já mostra o erro.
  if (loaded.kind !== "ok") return null;
  const viewer = loaded.tenant.role;

  return (
    <div id={MEMBERS_ID} tabIndex={-1}>
      <DataTable
        label={t("label")}
        description={t("count", { count: members.length })}
        columns={[
          { key: "member", header: t("member") },
          { key: "role", header: t("role") },
          { key: "since", header: t("since"), secondary: true },
        ]}
        rows={members.map((member) => {
          const changeRole = viewer === "owner" && member.role !== "owner";
          const remove = canRemove(viewer, member.role);
          return {
            id: String(member.user_id),
            href: `/${slug}/members/${member.user_id}`,
            search: `${member.nickname} ${member.display_name}`,
            cells: {
              member: <MemberName member={member} />,
              role: (
                <StatusTag status="neutral">{tRoles(member.role)}</StatusTag>
              ),
              since: <MemberSince value={member.joined_at} />,
            },
            actions:
              changeRole || remove ? (
                <MemberActions
                  slug={slug}
                  member={member}
                  canChangeRole={changeRole}
                  canRemove={remove}
                  focusId={MEMBERS_ID}
                />
              ) : undefined,
          };
        })}
        empty={{ text: t("empty") }}
      />
    </div>
  );
}
