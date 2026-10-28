import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { MemberList } from "@/features/members/components/member-list";
import { TabLoading, TableSkeleton } from "@/shared/ui/skeleton";

/** Aba Membros da comunidade. */
export default async function TenantMembersPage({
  params,
}: PageProps<"/[locale]/[tenant]/members">) {
  const t = await getTranslations("Tenant.members");
  return (
    <Suspense
      fallback={
        <TabLoading label={t("loading")}>
          <TableSkeleton columns={3} />
        </TabLoading>
      }
    >
      {params.then(({ tenant }) => (
        <MemberList slug={tenant} />
      ))}
    </Suspense>
  );
}
