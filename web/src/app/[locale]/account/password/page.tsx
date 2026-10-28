import { Stack } from "@carbon/react";
import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { ChangePasswordSection } from "@/features/auth/components/change-password-section";
import { FormSkeleton } from "@/shared/ui/skeleton";

/** Aba Senha da conta: trocar o nome de usuário e a senha. */
export default async function AccountPasswordPage() {
  const t = await getTranslations("Account");
  return (
    <Stack gap={7}>
      <Suspense
        fallback={<FormSkeleton heading text fields={3} label={t("loading")} />}
      >
        <ChangePasswordSection />
      </Suspense>
    </Stack>
  );
}
