import { getTranslations } from "next-intl/server";

import { PasswordResetForm } from "@/features/auth/components/password-reset-form";
import { Page } from "@/shared/ui/page";
import { PageHeader } from "@/shared/ui/page-header";

/** O link do email de recuperação (`#token`): a senha nova, e entra. */
export default async function PasswordResetPage() {
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
        title={t("reset.title")}
        meta={t("reset.text")}
      />
      <PasswordResetForm />
    </Page>
  );
}
