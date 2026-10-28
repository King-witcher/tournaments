import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { MemberList } from "@/features/members/components/member-list";
import { TenantTabSkeleton } from "@/features/tenants/components/tenant-tab-skeleton";
import { TableSkeleton } from "@/shared/ui/skeleton";

/** Aba Membros da comunidade. */
export default async function TenantMembersPage({
  params,
}: PageProps<"/[locale]/[tenant]/members">) {
  const t = await getTranslations("Tenant.members");
  return (
    <Suspense
      fallback={
        <TenantTabSkeleton label={t("loading")}>
          <TableSkeleton columns={3} />
        </TenantTabSkeleton>
      }
    >
      {params.then(({ tenant }) => (
        <MemberList slug={tenant} />
      ))}
    </Suspense>
  );
}
