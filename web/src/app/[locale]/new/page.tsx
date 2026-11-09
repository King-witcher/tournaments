import { getTranslations } from "next-intl/server";
import { Suspense } from "react";
import { RequireLogin } from "@/features/auth/components/require-login";
import { CreateTenantForm } from "@/features/tenants/components/create-tenant-form";
import { Page } from "@/shared/ui/page";
import { PageHeader } from "@/shared/ui/page-header";
import { Panel } from "@/shared/ui/panel";
import { FormSkeleton } from "@/shared/ui/skeleton";

export default async function NewTenantPage() {
  const t = await getTranslations("NewTenant");
  return (
    <Page>
      <PageHeader title={t("title")} />
      <Suspense
        fallback={
          <Panel>
            <FormSkeleton label={false} />
          </Panel>
        }
      >
        <RequireLogin message={t("loginRequired")}>
          <Panel>
            <CreateTenantForm />
          </Panel>
        </RequireLogin>
      </Suspense>
    </Page>
  );
}
