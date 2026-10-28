import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { AccountTab } from "@/features/auth/components/account-tab";
import { ChangePasswordSection } from "@/features/auth/components/change-password-section";
import { FormSkeleton } from "@/features/auth/components/form-skeleton";

/** Aba Senha da conta: trocar o nome de usuário e a senha. */
export default async function AccountPasswordPage() {
  const t = await getTranslations("Account");
  return (
    <AccountTab>
      <Suspense
        fallback={<FormSkeleton heading text fields={3} label={t("loading")} />}
      >
        <ChangePasswordSection />
      </Suspense>
    </AccountTab>
  );
}
