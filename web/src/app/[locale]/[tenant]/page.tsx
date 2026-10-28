import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { TenantView } from "@/features/tenants/components/tenant-view";
import { Page } from "@/shared/ui/page";
import { PageSkeleton } from "@/shared/ui/skeleton";

export default async function TenantPage({
  params,
}: PageProps<"/[locale]/[tenant]">) {
  const t = await getTranslations("Tenant");
  return (
    <Page spacing="loose">
      <Suspense fallback={<PageSkeleton label={t("loading")} />}>
        {params.then(({ tenant }) => (
          <TenantView slug={tenant} />
        ))}
      </Suspense>
    </Page>
  );
}
