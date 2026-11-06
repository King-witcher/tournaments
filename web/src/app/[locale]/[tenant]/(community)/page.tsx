import { Suspense } from "react";

import { TournamentList } from "@/features/tournaments/components/tournament-list";
import { TournamentListSkeleton } from "@/features/tournaments/components/tournament-list-skeleton";

/** Aba Torneios da comunidade. */
export default function TenantTournamentsPage({
  params,
}: PageProps<"/[locale]/[tenant]">) {
  return (
    <Suspense fallback={<TournamentListSkeleton />}>
      {params.then(({ tenant }) => (
        <TournamentList slug={tenant} />
      ))}
    </Suspense>
  );
}
