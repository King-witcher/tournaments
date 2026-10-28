import { getTranslations } from "next-intl/server";

import { EmailConfirm } from "@/features/auth/components/email-confirm";
import { Page } from "@/shared/ui/page";
import { PageHeader } from "@/shared/ui/page-header";

/**
 * O link do email mandado ao trocar o email da conta (`#token`): confirmar
 * troca o email. Não exige login. O segmento `email` é reservado na API, para
 * não colidir com o slug de uma comunidade (`[tenant]`).
 */
export default async function EmailConfirmPage() {
  const [t, tAccount, tNav] = await Promise.all([
    getTranslations("Account.emailConfirm"),
    getTranslations("Account"),
    getTranslations("Nav"),
  ]);
  return (
    <Page>
      <PageHeader
        breadcrumbs={[
          { href: "/", label: tNav("home") },
          { href: "/account", label: tAccount("title") },
        ]}
        title={t("title")}
        meta={t("text")}
      />
      <EmailConfirm />
    </Page>
  );
}
