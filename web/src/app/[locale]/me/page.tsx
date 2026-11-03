import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { AnonymousIntro } from "@/features/auth/components/anonymous-intro";
import { AuthError } from "@/features/auth/components/auth-error";
import { ApiStatus } from "@/features/health/components/api-status";
import { WaitingForYou } from "@/features/organizing/components/waiting-for-you";
import { LiveUpdates } from "@/features/realtime/components/live-updates";
import { MyTenants } from "@/features/tenants/components/my-tenants";
import { NoTenants } from "@/features/tenants/components/no-tenants";
import { MyTournaments } from "@/features/tournaments/components/my-tournaments";
import { OpenTournaments } from "@/features/tournaments/components/open-tournaments";
import { VetoRoomEntry } from "@/features/veto-rooms/components/veto-room-entry";
import { Page } from "@/shared/ui/page";
import { PageColumns } from "@/shared/ui/page-columns";
import { PageHeader } from "@/shared/ui/page-header";
import { SectionSkeleton } from "@/shared/ui/skeleton";

/** Alvo do foco quando a ação usada some da página (`tabIndex={-1}`). */
const TITLE_ID = "home-title";

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
      <PageHeader
        title={
          <span id={TITLE_ID} tabIndex={-1}>
            {t("title")}
          </span>
        }
      />
      {/* "Esperando você" e "Seus torneios" recarregam sozinhos. */}
      <LiveUpdates me />
      <Suspense fallback={null}>
        <ApiStatus />
      </Suspense>
      {/* Uma espera só: quem não entrou vê o convite a entrar, quem entrou vê as
          listas, e a página troca o esqueleto uma vez em vez de pular três. */}
      <Suspense
        fallback={
          <PageColumns
            main={<SectionSkeleton tiles={3} />}
            aside={<SectionSkeleton tiles={2} />}
          />
        }
      >
        <AnonymousIntro />
        <PageColumns
          main={
            <>
              <WaitingForYou focusAfter={TITLE_ID} />
              <NoTenants />
              <MyTournaments />
              <OpenTournaments />
            </>
          }
          aside={
            <>
              <MyTenants />
              <VetoRoomEntry />
            </>
          }
        />
      </Suspense>
    </Page>
  );
}
