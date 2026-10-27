import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { TenantView } from "@/features/tenants/components/tenant-view";
import { Page } from "@/shared/ui/page";

export default async function TenantPage({
  params,
}: PageProps<"/[locale]/[tenant]">) {
  const t = await getTranslations("Tenant");
  return (
    <Page spacing="loose">
      <Suspense fallback={<p>{t("loading")}</p>}>
        {params.then(({ tenant }) => (
          <TenantView slug={tenant} />
        ))}
      </Suspense>
    </Page>
  );
}
