import { Suspense } from "react";

import { MemberOverview } from "@/features/members/components/member-overview";
import { OverviewSkeleton } from "@/features/members/components/overview-skeleton";

/** A Visão geral da pessoa. */
export default function MemberOverviewPage({
  params,
}: PageProps<"/[locale]/[tenant]/members/[user]">) {
  return (
    <Suspense fallback={<OverviewSkeleton />}>
      {params.then(({ tenant, user }) => (
        <MemberOverview tenant={tenant} user={user} />
      ))}
    </Suspense>
  );
}
