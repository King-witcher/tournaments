import { getTranslations } from "next-intl/server";

import { SignupConfirm } from "@/features/auth/components/signup-confirm";
import { Page } from "@/shared/ui/page";
import { PageHeader } from "@/shared/ui/page-header";

/** O link do email de cadastro (`#token`): confirmar cria a conta e entra. */
export default async function SignupConfirmPage() {
  const [t, tNav] = await Promise.all([
    getTranslations("Signup"),
    getTranslations("Nav"),
  ]);
  return (
    <Page>
      <PageHeader
        breadcrumbs={[
          { href: "/", label: tNav("home") },
          { href: "/signup", label: t("title") },
        ]}
        title={t("confirm.title")}
        meta={t("confirm.text")}
      />
      <SignupConfirm />
    </Page>
  );
}
