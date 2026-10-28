import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { InviteSection } from "@/features/invites/components/invite-section";
import { TenantTabSkeleton } from "@/features/tenants/components/tenant-tab-skeleton";
import { TableSkeleton } from "@/shared/ui/skeleton";

/** Aba Convites da comunidade (organização). */
export default async function TenantInvitesPage({
  params,
}: PageProps<"/[locale]/[tenant]/invites">) {
  const t = await getTranslations("Tenant.invites");
  return (
    <Suspense
      fallback={
        <TenantTabSkeleton label={t("loading")}>
          <TableSkeleton columns={4} rows={3} />
        </TenantTabSkeleton>
      }
    >
      {params.then(({ tenant }) => (
        <InviteSection slug={tenant} />
      ))}
    </Suspense>
  );
}
