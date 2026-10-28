import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { MembershipSettings } from "@/features/members/components/membership-settings";
import { SectionSkeleton, TabLoading } from "@/shared/ui/skeleton";

/** Aba Configurações da comunidade: o que é de quem vê. */
export default async function TenantSettingsPage({
  params,
}: PageProps<"/[locale]/[tenant]/settings">) {
  const t = await getTranslations("Tenant.settings");
  return (
    <Suspense
      fallback={
        <TabLoading label={t("loading")}>
          <SectionSkeleton rows={2} />
          <SectionSkeleton rows={1} />
        </TabLoading>
      }
    >
      {params.then(({ tenant }) => (
        <MembershipSettings slug={tenant} />
      ))}
    </Suspense>
  );
}
