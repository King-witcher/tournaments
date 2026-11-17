import { Suspense } from "react";

import { GamesSkeleton } from "@/features/members/components/games-skeleton";
import { MemberGamesTab } from "@/features/members/components/member-games-tab";
import { parseGamesQuery } from "@/features/members/lib/games-href";

/** As partidas da pessoa, com filtro e paginação no endereço. */
export default function MemberGamesPage({
  params,
  searchParams,
}: PageProps<"/[locale]/[tenant]/members/[user]/games">) {
  return (
    <Suspense fallback={<GamesSkeleton />}>
      {Promise.all([params, searchParams]).then(
        ([{ tenant, user }, search]) => (
          <MemberGamesTab
            tenant={tenant}
            user={user}
            query={parseGamesQuery(search)}
          />
        ),
      )}
    </Suspense>
  );
}
