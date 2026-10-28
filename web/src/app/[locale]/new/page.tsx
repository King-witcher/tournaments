import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { RequireLogin } from "@/features/auth/components/require-login";
import { CreateTenantForm } from "@/features/tenants/components/create-tenant-form";
import { NewTenantFormSkeleton } from "@/features/tenants/components/new-tenant-form-skeleton";
import { Page } from "@/shared/ui/page";
import { PageHeader } from "@/shared/ui/page-header";

export default async function NewTenantPage() {
  const [t, tNav] = await Promise.all([
    getTranslations("NewTenant"),
    getTranslations("Nav"),
  ]);
  return (
    <Page>
      <PageHeader
        breadcrumbs={[{ href: "/", label: tNav("home") }]}
        title={t("title")}
        meta={t("limit")}
      />
      <Suspense fallback={<NewTenantFormSkeleton />}>
        <RequireLogin message={t("loginRequired")}>
          <CreateTenantForm />
        </RequireLogin>
      </Suspense>
    </Page>
  );
}
