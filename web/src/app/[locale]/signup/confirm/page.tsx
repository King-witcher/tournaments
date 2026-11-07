import { getTranslations } from "next-intl/server";

import { SignupConfirm } from "@/features/auth/components/signup-confirm";
import { Page } from "@/shared/ui/page";
import { PageHeader } from "@/shared/ui/page-header";

/** O link do email de cadastro (`#token`): o botão que cria a conta e entra. */
export default async function SignupConfirmPage() {
  const t = await getTranslations("Signup");
  return (
    <Page>
      <PageHeader
        breadcrumbs={[{ href: "/signup", label: t("title") }]}
        title={t("confirm.title")}
      />
      <SignupConfirm />
    </Page>
  );
}
