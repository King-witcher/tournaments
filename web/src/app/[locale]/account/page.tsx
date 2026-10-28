import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { AccountLogins } from "@/features/auth/components/account-logins";
import { AuthError } from "@/features/auth/components/auth-error";
import { RequireLogin } from "@/features/auth/components/require-login";
import { Page } from "@/shared/ui/page";
import { LoadingSection } from "@/shared/ui/skeleton";

export default async function AccountPage({
  searchParams,
}: PageProps<"/[locale]/account">) {
  const t = await getTranslations("Account");
  return (
    <Page spacing="loose">
      <h1 className="text-3xl font-semibold">{t("title")}</h1>
      <Suspense fallback={null}>
        <AuthError searchParams={searchParams} />
      </Suspense>
      <Suspense fallback={<LoadingSection label={t("loading")} />}>
        <RequireLogin message={t("loginRequired")}>
          <AccountLogins />
        </RequireLogin>
      </Suspense>
    </Page>
  );
}
