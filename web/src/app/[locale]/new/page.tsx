import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { RequireLogin } from "@/features/auth/components/require-login";
import { CreateTenantForm } from "@/features/tenants/components/create-tenant-form";
import { Page } from "@/shared/ui/page";
import { PageHeader } from "@/shared/ui/page-header";
import { FormSkeleton } from "@/shared/ui/skeleton";

export default async function NewTenantPage() {
  const t = await getTranslations("NewTenant");
  return (
    <Page>
      <PageHeader title={t("title")} />
      <Suspense fallback={<FormSkeleton label={false} />}>
        <RequireLogin message={t("loginRequired")}>
          <CreateTenantForm />
        </RequireLogin>
      </Suspense>
    </Page>
  );
}
