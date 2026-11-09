import { getTranslations } from "next-intl/server";

import { InviteLinkSettings } from "@/features/invites/components/invite-link-settings";
import { serverApi } from "@/shared/api/server";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import { getTenant } from "../server";

/**
 * Nas configurações da comunidade, para organizer e owner: o link de convite,
 * o código e "Gerar novo link". Some com a exclusão agendada (ninguém mais
 * entra) e sem o token (a API falhou). Fica em `<Suspense>`.
 */
export async function TenantInviteSection({ slug }: { slug: string }) {
  const loaded = await getTenant(slug);
  if (loaded.kind !== "ok") return null;
  const { tenant } = loaded;
  if (tenant.role === "player" || tenant.deletion_scheduled_for) return null;

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
    <Panel>
      <Section title={t("link")} description={t("note")}>
        <InviteLinkSettings
          tenant={slug}
          token={token}
          linkLabel={t("link")}
          rotateConfirm={t("rotateConfirm")}
        />
      </Section>
    </Panel>
  );
}
