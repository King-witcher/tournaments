import { Stack } from "@carbon/react";
import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { AccountLogins } from "@/features/auth/components/account-logins";
import { AuthError } from "@/features/auth/components/auth-error";
import { LoginsSkeleton } from "@/features/auth/components/logins-skeleton";

/**
 * Aba Logins da conta: Discord e nome de usuário com senha. As props tipadas
 * à mão: o `PageProps` desta rota só existe depois que o Next regenera os
 * tipos das rotas (`next dev`/`next build`).
 */
export default async function AccountLoginsPage({
  searchParams,
}: {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
}) {
  const t = await getTranslations("Account");
  return (
    <Stack gap={7}>
      {/* Vincular o Discord volta para cá; recusado, com ?auth_error=. */}
      <Suspense fallback={null}>
        <AuthError searchParams={searchParams} />
      </Suspense>
      <Suspense fallback={<LoginsSkeleton label={t("loading")} />}>
        <AccountLogins />
      </Suspense>
    </Stack>
  );
}
