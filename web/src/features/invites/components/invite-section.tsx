import { getFormatter, getTranslations } from "next-intl/server";

import { getTenant } from "@/features/tenants/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { serverApi } from "@/shared/api/server";
import { DataTable } from "@/shared/ui/data-table";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { PageError } from "@/shared/ui/page-error";
import { CreateInviteButton } from "./create-invite-button";
import { CreatedInviteProvider } from "./created-invite-provider";
import { InviteActions } from "./invite-actions";

/** A tabela, que recebe o foco quando um link revogado some dela. */
const INVITES_ID = "invites";

/**
 * A aba Convites (organização): os links ativos (quem criou, validade,
 * quantos entraram), "Gerar link" na barra e revogar por linha. Jogador que
 * abre o endereço direto vê o erro de permissão. Fica em `<Suspense>`.
 */
export async function InviteSection({ slug }: { slug: string }) {
  const api = await serverApi();
  const [loaded, t, tNav, format, invites] = await Promise.all([
    getTenant(slug),
    getTranslations("Tenant.invites"),
    getTranslations("Nav"),
    getFormatter(),
    api
      .GET("/tenants/{slug}/invites", { params: { path: { slug } } })
      .then(({ data }) => data ?? []),
  ]);
  // Sem acesso, o cabeçalho da comunidade já mostra o erro.
  if (loaded.kind !== "ok") return null;
  if (loaded.tenant.role === "player") {
    return (
      <PageError href={`/${slug}`} label={tNav("backToCommunity")}>
        {await apiErrorMessage("tenant.forbidden")}
      </PageError>
    );
  }
  const now = new Date();

  return (
    <CreatedInviteProvider>
      <div id={INVITES_ID} tabIndex={-1}>
        <DataTable
          label={t("label")}
          description={t("description")}
          columns={[
            { key: "createdBy", header: t("createdBy") },
            { key: "expires", header: t("expires") },
            { key: "uses", header: t("uses"), secondary: true },
            { key: "createdAt", header: t("createdAt"), secondary: true },
          ]}
          rows={invites.map((invite) => ({
            id: String(invite.id),
            search: invite.created_by,
            cells: {
              createdBy: invite.created_by,
              expires: format.relativeTime(new Date(invite.expires_at), now),
              uses: t("usesCount", { count: invite.uses }),
              createdAt: <LocalDateTime value={invite.created_at} />,
            },
            actions: (
              <InviteActions
                slug={slug}
                inviteId={invite.id}
                createdBy={invite.created_by}
                focusId={INVITES_ID}
              />
            ),
          }))}
          toolbar={<CreateInviteButton slug={slug} />}
          empty={{ text: t("empty") }}
        />
      </div>
    </CreatedInviteProvider>
  );
}
