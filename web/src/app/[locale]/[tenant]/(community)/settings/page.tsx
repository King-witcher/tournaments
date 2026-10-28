import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { MembershipSettings } from "@/features/members/components/membership-settings";
import { TenantTabSkeleton } from "@/features/tenants/components/tenant-tab-skeleton";
import { SectionSkeleton } from "@/shared/ui/skeleton";

/** Aba Configurações da comunidade: o que é de quem vê. */
export default async function TenantSettingsPage({
  params,
}: PageProps<"/[locale]/[tenant]/settings">) {
  const t = await getTranslations("Tenant.settings");
  return (
    <Suspense
      fallback={
        <TenantTabSkeleton label={t("loading")}>
          <SectionSkeleton rows={2} />
          <SectionSkeleton rows={1} />
        </TenantTabSkeleton>
      }
    >
      {params.then(({ tenant }) => (
        <MembershipSettings slug={tenant} />
      ))}
    </Suspense>
  );
}
