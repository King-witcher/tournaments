import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { MemberTournamentsTab } from "@/features/members/components/member-tournaments-tab";
import { TabLoading, TableSkeleton } from "@/shared/ui/skeleton";

/** Os torneios da comunidade em que a pessoa se inscreveu. */
export default async function MemberTournamentsPage({
  params,
}: PageProps<"/[locale]/[tenant]/members/[user]/tournaments">) {
  const t = await getTranslations("Member.loadingTab");
  return (
    <Suspense
      fallback={
        <TabLoading label={t("tournaments")}>
          <TableSkeleton columns={4} rows={5} />
        </TabLoading>
      }
    >
      {params.then(({ tenant, user }) => (
        <MemberTournamentsTab tenant={tenant} user={user} />
      ))}
    </Suspense>
  );
}
