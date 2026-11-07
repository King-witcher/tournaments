import { Suspense } from "react";

import { LiveUpdates } from "@/features/realtime/components/live-updates";
import { Page } from "@/shared/ui/page";

/**
 * As seções da comunidade (Torneios, Ranking, Membros, Configurações), que o
 * menu lateral alterna: a página larga e o tempo real. Cada seção desenha o
 * próprio cabeçalho (`TenantHeader`), com o título dela e a ação primária.
 */
export default function CommunityLayout({
  params,
  children,
}: LayoutProps<"/[locale]/[tenant]">) {
  return (
    <Page width="wide">
      <Suspense fallback={null}>
        {params.then(({ tenant }) => (
          <LiveUpdates tenant={tenant} />
        ))}
      </Suspense>
      {children}
    </Page>
  );
}
