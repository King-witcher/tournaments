import { getTranslations } from "next-intl/server";

import { EmailLoginConfirm } from "@/features/auth/components/email-login-confirm";
import { Page } from "@/shared/ui/page";
import { PageHeader } from "@/shared/ui/page-header";

/** O link do email de entrar (`#token`): o botão de entrar. */
export default async function EmailLoginConfirmPage() {
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
        title={t("email.title")}
      />
      <EmailLoginConfirm />
    </Page>
  );
}
