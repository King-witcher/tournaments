import { getTranslations } from "next-intl/server";

import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { MemberRow } from "./member-row";

type Props = {
  slug: string;
  /** Papel de quem está vendo: o owner troca o papel dos outros. */
  viewerRole: components["schemas"]["Role"];
};

/** Membros do tenant. Fica em `<Suspense>`. */
export async function MemberList({ slug, viewerRole }: Props) {
  const api = await serverApi();
  const [t, members] = await Promise.all([
    getTranslations("Tenant.members"),
    api
      .GET("/tenants/{slug}/members", { params: { path: { slug } } })
      .then(({ data }) => data ?? []),
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
          />
        ))}
      </ul>
    </section>
  );
}
