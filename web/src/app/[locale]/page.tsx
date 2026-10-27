import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { AuthError } from "@/features/auth/components/auth-error";
import { ApiStatus } from "@/features/health/components/api-status";
import { MyTenants } from "@/features/tenants/components/my-tenants";
import { Page } from "@/shared/ui/page";

export default async function Home({ searchParams }: PageProps<"/[locale]">) {
  const t = await getTranslations("Home");

  return (
    <Page>
      <Suspense fallback={null}>
        <AuthError searchParams={searchParams} />
      </Suspense>
      <h1 className="text-3xl font-semibold">{t("title")}</h1>
      <p>{t("subtitle")}</p>
      <Suspense fallback={<p>{t("status.checking")}</p>}>
        <ApiStatus />
      </Suspense>
      <Suspense fallback={null}>
        <MyTenants />
      </Suspense>
    </Page>
  );
}
