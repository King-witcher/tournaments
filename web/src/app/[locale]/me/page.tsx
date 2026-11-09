import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { AuthError } from "@/features/auth/components/auth-error";
import { BackupLoginReminder } from "@/features/auth/components/backup-login-reminder";
import { HomeColumns } from "@/features/auth/components/home-columns";
import { HomeSessionGate } from "@/features/auth/components/home-session-gate";
import { WaitingForYou } from "@/features/organizing/components/waiting-for-you";
import { LiveUpdates } from "@/features/realtime/components/live-updates";
import { MyTenants } from "@/features/tenants/components/my-tenants";
import { NoTenants } from "@/features/tenants/components/no-tenants";
import { MyTournaments } from "@/features/tournaments/components/my-tournaments";
import { OpenTournaments } from "@/features/tournaments/components/open-tournaments";
import { VetoRoomEntry } from "@/features/veto-rooms/components/veto-room-entry";
import { Page } from "@/shared/ui/page";
import { PageHeader } from "@/shared/ui/page-header";
import { SectionSkeleton, TaskNoticesSkeleton } from "@/shared/ui/skeleton";

/** Alvo do foco quando a ação usada some da página (`tabIndex={-1}`). */
const TITLE_ID = "home-title";

const SKELETON = (
  <HomeColumns
    lead={<TaskNoticesSkeleton />}
    aside={<SectionSkeleton tiles={2} />}
  />
);

/**
 * O início de quem entrou. O proxy a serve em `/` quando há cookie de
 * sessão; sem ele, `/` é a apresentação do site (`(home)`). Sem menu lateral:
 * é o nível acima das comunidades, uma página só, e as comunidades já estão
 * no cabeçalho (`TenantSwitcher`) e em "Suas comunidades".
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
        actions={<VetoRoomEntry />}
      />
      {/* As pendências e "Seus torneios" recarregam sozinhos. */}
      <LiveUpdates me />
      {/* Uma espera só: a página troca o esqueleto uma vez em vez de pular três. */}
      <Suspense fallback={SKELETON}>
        <HomeSessionGate pending={SKELETON}>
          <BackupLoginReminder />
          <HomeColumns
            lead={<WaitingForYou focusAfter={TITLE_ID} />}
            aside={<MyTenants />}
            main={
              <>
                <NoTenants />
                <MyTournaments />
                <OpenTournaments />
              </>
            }
          />
        </HomeSessionGate>
      </Suspense>
    </Page>
  );
}
