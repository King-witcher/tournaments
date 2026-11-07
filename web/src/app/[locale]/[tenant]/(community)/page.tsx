import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { WaitingForYou } from "@/features/organizing/components/waiting-for-you";
import { RankingTop } from "@/features/ratings/components/ranking-top";
import { TenantAbout } from "@/features/tenants/components/tenant-about";
import { TenantHeader } from "@/features/tenants/components/tenant-header";
import { TENANT_TITLE_ID } from "@/features/tenants/lib/focus";
import { NewTournamentAction } from "@/features/tournaments/components/new-tournament-action";
import { TournamentList } from "@/features/tournaments/components/tournament-list";
import { TournamentListSkeleton } from "@/features/tournaments/components/tournament-list-skeleton";
import { PageColumns } from "@/shared/ui/page-columns";
import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";

/**
 * O início da comunidade, a seção Torneios: na coluna principal o que espera
 * quem vê aqui e os torneios (o único lugar que lista todos); na estreita, a
 * descrição e o topo do ranking.
 */
export default async function TenantTournamentsPage({
  params,
}: PageProps<"/[locale]/[tenant]">) {
  const [t, tList] = await Promise.all([
    getTranslations("Tenant"),
    getTranslations("Tournaments"),
  ]);
  return (
    <>
      <Suspense fallback={<PageHeaderSkeleton label={tList("loading")} />}>
        {params.then(({ tenant }) => (
          <TenantHeader
            slug={tenant}
            title={t("tabs.tournaments")}
            home
            actions={<NewTournamentAction slug={tenant} />}
          />
        ))}
      </Suspense>
      <PageColumns
        main={
          // Uma espera só: "Esperando você" (que some sem nada) não entra
          // depois, empurrando a lista.
          <Suspense fallback={<TournamentListSkeleton />}>
            {params.then(({ tenant }) => (
              <>
                <WaitingForYou tenant={tenant} focusAfter={TENANT_TITLE_ID} />
                <TournamentList slug={tenant} />
              </>
            ))}
          </Suspense>
        }
        aside={
          <Suspense fallback={<SectionSkeleton rows={3} />}>
            {params.then(({ tenant }) => (
              <>
                <TenantAbout slug={tenant} />
                <RankingTop slug={tenant} />
              </>
            ))}
          </Suspense>
        }
      />
    </>
  );
}
