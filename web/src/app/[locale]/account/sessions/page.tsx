import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { AccountSessions } from "@/features/auth/components/account-sessions";
import { SessionsSkeleton } from "@/features/auth/components/sessions-skeleton";
import { PageHeader } from "@/shared/ui/page-header";

/**
 * As Sessões da conta: onde ela está aberta e como encerrar, na largura
 * toda, como as listas da comunidade.
 */
export default async function AccountSessionsPage() {
  const t = await getTranslations("Account");
  return (
    <>
      <PageHeader title={t("tabs.sessions")} />
      <Suspense fallback={<SessionsSkeleton label={t("loading")} />}>
        <AccountSessions />
      </Suspense>
    </>
  );
}
