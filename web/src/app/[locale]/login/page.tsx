import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { LoginView } from "@/features/auth/components/login-view";
import { Page } from "@/shared/ui/page";
import { LoadingSection } from "@/shared/ui/skeleton";

export default async function LoginPage({
  searchParams,
}: PageProps<"/[locale]/login">) {
  const t = await getTranslations("Login");
  return (
    <Page>
      <h1 className="text-3xl font-semibold">{t("title")}</h1>
      <Suspense fallback={<LoadingSection rows={2} />}>
        <LoginView searchParams={searchParams} />
      </Suspense>
    </Page>
  );
}
