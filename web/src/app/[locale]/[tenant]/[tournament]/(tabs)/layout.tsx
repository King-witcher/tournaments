import { getTranslations } from "next-intl/server";
import { Suspense } from "react";
import { LiveUpdates } from "@/features/realtime/components/live-updates";
import { TournamentView } from "@/features/tournaments/components/tournament-view";
import { Page } from "@/shared/ui/page";
import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";

/**
 * O torneio: o cabeçalho (o completo na Visão geral, o compacto nas outras
 * seções) e a seção aberta. As seções (Visão geral, Chave, Classificação,
 * Inscritos, Regras, Configurações) ficam no menu lateral. A
 * ordem da chave, os times e a partida ficam fora do grupo.
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
            <PageHeaderSkeleton breadcrumb={false} meta label={t("loading")} />
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
