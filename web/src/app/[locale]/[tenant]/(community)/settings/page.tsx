import { getTranslations } from "next-intl/server";
import { Suspense } from "react";
import { CommunityDangerZone } from "@/features/members/components/community-danger-zone";
import { TenantHeader } from "@/features/tenants/components/tenant-header";
import { TenantSettingsGeneral } from "@/features/tenants/components/tenant-settings-general";
import { Stack } from "@/shared/ui/carbon/stack";
import { Panel } from "@/shared/ui/panel";
import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";

const SKELETON = (
  <Stack gap={7}>
    <Panel>
      <SectionSkeleton rows={3} />
    </Panel>
    <Panel>
      <SectionSkeleton rows={1} />
    </Panel>
  </Stack>
);

/**
 * Geral, a primeira página das Configurações da comunidade e a única do
 * organizer (o jogador não tem Configurações): o que o owner decide sobre
 * ela, o link de convite e, por último, a zona de perigo do owner. Rating e
 * Discord têm a página deles.
 */
export default async function TenantSettingsPage({
  params,
}: PageProps<"/[locale]/[tenant]/settings">) {
  const t = await getTranslations("Tenant");
  return (
    <>
      <Suspense
        fallback={
          <PageHeaderSkeleton
            breadcrumb="belowLg"
            label={t("settings.loading")}
          />
        }
      >
        {params.then(({ tenant }) => (
          <TenantHeader slug={tenant} title={t("tabs.settings")} settings />
        ))}
      </Suspense>
      <Suspense fallback={SKELETON}>
        {params.then(({ tenant }) => (
          <Stack gap={7}>
            <TenantSettingsGeneral slug={tenant} />
            <CommunityDangerZone slug={tenant} />
          </Stack>
        ))}
      </Suspense>
    </>
  );
}
