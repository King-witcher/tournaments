import { getTranslations } from "next-intl/server";
import { Suspense } from "react";
import { CommunityDangerZone } from "@/features/members/components/community-danger-zone";
import { NicknameSection } from "@/features/members/components/nickname-section";
import { TenantGeneralSection } from "@/features/tenants/components/tenant-general-section";
import { TenantHeader } from "@/features/tenants/components/tenant-header";
import { TenantInviteSection } from "@/features/tenants/components/tenant-invite-section";
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
 * Geral, a primeira página das Configurações da comunidade e a única de quem
 * não é owner: o que o owner decide sobre ela (nome, endereço, descrição e
 * logo), o link de convite (organização), o apelido de quem vê e, por último,
 * a zona de perigo. Rating e Discord têm a página deles.
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
            <TenantGeneralSection slug={tenant} />
            <TenantInviteSection slug={tenant} />
            <NicknameSection slug={tenant} />
            <CommunityDangerZone slug={tenant} />
          </Stack>
        ))}
      </Suspense>
    </>
  );
}
