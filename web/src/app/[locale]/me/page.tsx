import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { AuthError } from "@/features/auth/components/auth-error";
import { BackupLoginReminder } from "@/features/auth/components/backup-login-reminder";
import { HomeSessionGate } from "@/features/auth/components/home-session-gate";
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

const SKELETON = (
  <PageColumns
    main={<SectionSkeleton tiles={3} />}
    aside={<SectionSkeleton tiles={2} />}
  />
);

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
      {/* Uma espera só: a página troca o esqueleto uma vez em vez de pular três. */}
      <Suspense fallback={SKELETON}>
        <HomeSessionGate pending={SKELETON}>
          <BackupLoginReminder />
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
        </HomeSessionGate>
      </Suspense>
    </Page>
  );
}
