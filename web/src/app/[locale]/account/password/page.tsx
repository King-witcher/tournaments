import { Stack } from "@carbon/react";
import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { ChangePasswordSection } from "@/features/auth/components/change-password-section";
import { PasswordResetDone } from "@/features/auth/components/password-reset-done";
import { FormSkeleton } from "@/shared/ui/skeleton";

/**
 * Aba Senha da conta: trocar o nome de usuário e a senha. As props tipadas à
 * mão: o `PageProps` desta rota só existe depois que o Next regenera os
 * tipos das rotas (`next dev`/`next build`).
 */
export default async function AccountPasswordPage({
  searchParams,
}: {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
}) {
  const t = await getTranslations("Account");
  return (
    <Stack gap={7}>
      {/* A recuperação pelo email volta para cá, com ?password_reset=1. */}
      <Suspense fallback={null}>
        <PasswordResetDone searchParams={searchParams} />
      </Suspense>
      <Suspense
        fallback={<FormSkeleton heading text fields={3} label={t("loading")} />}
      >
        <ChangePasswordSection />
      </Suspense>
    </Stack>
  );
}
