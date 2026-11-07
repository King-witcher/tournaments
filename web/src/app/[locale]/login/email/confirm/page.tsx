import { getTranslations } from "next-intl/server";

import { EmailLoginConfirm } from "@/features/auth/components/email-login-confirm";
import { Page } from "@/shared/ui/page";
import { PageHeader } from "@/shared/ui/page-header";

/** O link do email de entrar (`#token`): o botão de entrar. */
export default async function EmailLoginConfirmPage() {
  const t = await getTranslations("Login");
  return (
    <Page>
      <PageHeader
        breadcrumbs={[{ href: "/login", label: t("title") }]}
        title={t("email.title")}
      />
      <EmailLoginConfirm />
    </Page>
  );
}
