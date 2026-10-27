import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { RequireLogin } from "@/features/auth/components/require-login";
import { CreateTenantForm } from "@/features/tenants/components/create-tenant-form";
import { Page } from "@/shared/ui/page";

export default async function NewTenantPage() {
  const t = await getTranslations("NewTenant");
  return (
    <Page>
      <h1 className="text-3xl font-semibold">{t("title")}</h1>
      <Suspense fallback={null}>
        <RequireLogin message={t("loginRequired")}>
          <CreateTenantForm />
        </RequireLogin>
      </Suspense>
    </Page>
  );
}
