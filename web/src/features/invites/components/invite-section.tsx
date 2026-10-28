import { getTranslations } from "next-intl/server";

import { serverApi } from "@/shared/api/server";
import { CreateInviteButton } from "./create-invite-button";
import { InviteRow } from "./invite-row";

/** Convites ativos do tenant, para organizer e owner. Fica em `<Suspense>`. */
export async function InviteSection({ slug }: { slug: string }) {
  const api = await serverApi();
  const [t, invites] = await Promise.all([
    getTranslations("Tenant.invites"),
    api
      .GET("/tenants/{slug}/invites", { params: { path: { slug } } })
      .then(({ data }) => data ?? []),
  ]);
  const now = new Date();

  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-xl font-semibold">{t("title")}</h2>
      <p className="text-sm text-muted">{t("description")}</p>
      <CreateInviteButton slug={slug} />
      {invites.length === 0 ? (
        <p className="text-sm text-muted">{t("empty")}</p>
      ) : (
        <ul className="divide-y divide-black/10 dark:divide-white/10">
          {invites.map((invite) => (
            <InviteRow key={invite.id} slug={slug} invite={invite} now={now} />
          ))}
        </ul>
      )}
    </section>
  );
}
