import { getTranslations } from "next-intl/server";

import { ForgotPasswordView } from "@/features/auth/components/forgot-password-view";
import { Page } from "@/shared/ui/page";
import { PageHeader } from "@/shared/ui/page-header";

/** Esqueci a senha: pede o link de recuperação ao email da conta. */
export default async function ForgotPasswordPage() {
  const [t, tNav] = await Promise.all([
    getTranslations("Login"),
    getTranslations("Nav"),
  ]);
  return (
    <Page>
      <PageHeader
        breadcrumbs={[
          { href: "/", label: tNav("home") },
          { href: "/login", label: t("title") },
        ]}
        title={t("forgot.title")}
        meta={t("forgot.text")}
      />
      <ForgotPasswordView />
    </Page>
  );
}
