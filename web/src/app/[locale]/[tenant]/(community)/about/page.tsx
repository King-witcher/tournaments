import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { TenantAbout } from "@/features/tenants/components/tenant-about";
import { SectionSkeleton, TabLoading } from "@/shared/ui/skeleton";

/** Aba Sobre da comunidade. */
export default async function TenantAboutPage({
  params,
}: PageProps<"/[locale]/[tenant]/about">) {
  const t = await getTranslations("Tenant.about");
  return (
    <Suspense
      fallback={
        <TabLoading label={t("loading")}>
          <SectionSkeleton rows={3} />
        </TabLoading>
      }
    >
      {params.then(({ tenant }) => (
        <TenantAbout slug={tenant} />
      ))}
    </Suspense>
  );
}
