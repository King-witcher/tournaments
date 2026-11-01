import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { AnonymousIntro } from "@/features/auth/components/anonymous-intro";
import { AuthError } from "@/features/auth/components/auth-error";
import { ApiStatus } from "@/features/health/components/api-status";
import { OrganizingPending } from "@/features/organizing/components/organizing-pending";
import { LiveUpdates } from "@/features/realtime/components/live-updates";
import { MyTenants } from "@/features/tenants/components/my-tenants";
import { MyTournaments } from "@/features/tournaments/components/my-tournaments";
import { Page } from "@/shared/ui/page";
import { PageHeader } from "@/shared/ui/page-header";
import { SectionSkeleton } from "@/shared/ui/skeleton";

/**
 * O início de quem entrou. O proxy a serve em `/` quando há cookie de
 * sessão; sem ele, `/` é a apresentação do site (`(home)`).
 */
export default async function SignedInHome({
  searchParams,
}: PageProps<"/[locale]/me">) {
  const t = await getTranslations("Home");

  return (
    <Page width="wide">
      <Suspense fallback={null}>
        <AuthError searchParams={searchParams} />
      </Suspense>
      <PageHeader title={t("title")} />
      {/* As pendências da organização, "Seus torneios" e "Suas comunidades"
          recarregam sozinhos. */}
      <LiveUpdates me />
      <Suspense fallback={null}>
        <ApiStatus />
      </Suspense>
      {/* Uma espera só: quem não entrou vê o convite a entrar, quem entrou vê as
          listas, e a página troca o esqueleto uma vez em vez de pular três. */}
      <Suspense
        fallback={
          <>
            <SectionSkeleton tiles={3} />
            <SectionSkeleton tiles={2} />
          </>
        }
      >
        <AnonymousIntro />
        <OrganizingPending />
        <MyTournaments />
        <MyTenants />
      </Suspense>
    </Page>
  );
}
