import { getTranslations } from "next-intl/server";
import { Suspense } from "react";
import { LiveUpdates } from "@/features/realtime/components/live-updates";
import { TournamentView } from "@/features/tournaments/components/tournament-view";
import { Page } from "@/shared/ui/page";
import {
  PageHeaderSkeleton,
  SectionSkeleton,
  TabsSkeleton,
} from "@/shared/ui/skeleton";

/**
 * O torneio: o cabeçalho, "Sua situação" e as abas (Visão geral, Chave,
 * Inscritos, Regras, Organização), com a aba aberta dentro. Editar, seeding
 * e a partida ficam fora do grupo, sem as abas.
 */
export default async function TournamentLayout({
  params,
  children,
}: LayoutProps<"/[locale]/[tenant]/[tournament]">) {
  const t = await getTranslations("Tournament");
  return (
    <Page width="wide">
      <Suspense
        fallback={
          <>
            <PageHeaderSkeleton meta label={t("loading")} />
            <TabsSkeleton />
            <SectionSkeleton rows={3} />
          </>
        }
      >
        {params.then(({ tenant, tournament }) => (
          <>
            <LiveUpdates tenant={tenant} tournament={tournament} />
            <TournamentView tenant={tenant} slug={tournament}>
              {children}
            </TournamentView>
          </>
        ))}
      </Suspense>
    </Page>
  );
}
