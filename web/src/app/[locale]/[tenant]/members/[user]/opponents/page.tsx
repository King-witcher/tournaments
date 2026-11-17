import { Suspense } from "react";

import { MemberOpponentsTab } from "@/features/members/components/member-opponents-tab";
import { OpponentsSkeleton } from "@/features/members/components/opponents-skeleton";
import { parseModality } from "@/features/members/lib/opponents-href";

/** O confronto direto da pessoa com cada adversário, com a modalidade no endereço. */
export default function MemberOpponentsPage({
  params,
  searchParams,
}: PageProps<"/[locale]/[tenant]/members/[user]/opponents">) {
  return (
    <Suspense fallback={<OpponentsSkeleton />}>
      {Promise.all([params, searchParams]).then(
        ([{ tenant, user }, search]) => (
          <MemberOpponentsTab
            tenant={tenant}
            user={user}
            modality={parseModality(search.modality)}
          />
        ),
      )}
    </Suspense>
  );
}
