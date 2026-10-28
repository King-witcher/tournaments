import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { AccountSessions } from "@/features/auth/components/account-sessions";
import { SessionsSkeleton } from "@/features/auth/components/sessions-skeleton";

/** Aba Sessões da conta: onde ela está aberta e como encerrar. */
export default async function AccountSessionsPage() {
  const t = await getTranslations("Account");
  return (
    <Suspense fallback={<SessionsSkeleton label={t("loading")} />}>
      <AccountSessions />
    </Suspense>
  );
}
