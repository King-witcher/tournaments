import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { AccountLogins } from "@/features/auth/components/account-logins";
import { AuthError } from "@/features/auth/components/auth-error";
import { LoginsSkeleton } from "@/features/auth/components/logins-skeleton";
import { PasswordResetDone } from "@/features/auth/components/password-reset-done";
import { PageHeader } from "@/shared/ui/page-header";

/**
 * Os Logins da conta: email e senha, Discord e Google (`AccountLogins`).
 * As props tipadas à mão: o `PageProps` desta rota só existe depois que o
 * Next regenera os tipos das rotas (`next dev`/`next build`).
 */
export default async function AccountLoginsPage({
  searchParams,
}: {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
}) {
  const t = await getTranslations("Account");
  return (
    <>
      <PageHeader title={t("tabs.logins")} />
      {/* Vincular um provedor volta para cá; recusado, com ?auth_error=. */}
      <Suspense fallback={null}>
        <AuthError searchParams={searchParams} />
      </Suspense>
      {/* Quem define uma senha nova ao entrar por email volta para cá, com ?password_reset=1. */}
      <Suspense fallback={null}>
        <PasswordResetDone searchParams={searchParams} />
      </Suspense>
      <Suspense fallback={<LoginsSkeleton label={t("loading")} />}>
        <AccountLogins />
      </Suspense>
    </>
  );
}
