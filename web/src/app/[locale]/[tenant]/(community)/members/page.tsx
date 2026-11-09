import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { MemberCount } from "@/features/members/components/member-count";
import { MemberInviteAction } from "@/features/members/components/member-invite-action";
import { MemberList } from "@/features/members/components/member-list";
import { TenantHeader } from "@/features/tenants/components/tenant-header";
import { PageHeaderSkeleton, TableSkeleton } from "@/shared/ui/skeleton";

/** Os Membros da comunidade: o cabeçalho com "Convidar" e a tabela. */
export default async function TenantMembersPage({
  params,
}: PageProps<"/[locale]/[tenant]/members">) {
  const t = await getTranslations("Tenant");
  return (
    <>
      <Suspense
        fallback={
          <PageHeaderSkeleton
            breadcrumb="belowLg"
            meta
            label={t("members.loading")}
          />
        }
      >
        {params.then(({ tenant }) => (
          <TenantHeader
            slug={tenant}
            title={t("tabs.members")}
            meta={<MemberCount slug={tenant} />}
            actions={<MemberInviteAction slug={tenant} />}
          />
        ))}
      </Suspense>
      <Suspense fallback={<TableSkeleton columns={3} />}>
        {params.then(({ tenant }) => (
          <MemberList slug={tenant} />
        ))}
      </Suspense>
    </>
  );
}
