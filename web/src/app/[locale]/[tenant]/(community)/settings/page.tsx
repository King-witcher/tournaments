import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { DiscordSection } from "@/features/discord/components/discord-section";
import { CommunityDangerZone } from "@/features/members/components/community-danger-zone";
import { NicknameSection } from "@/features/members/components/nickname-section";
import { RatingSettingsSection } from "@/features/ratings/components/rating-settings-section";
import { CommunitySettingsColumns } from "@/features/tenants/components/community-settings-columns";
import { LogoSection } from "@/features/tenants/components/logo-section";
import { TenantAddressSection } from "@/features/tenants/components/tenant-address-section";
import { TenantDetailsSection } from "@/features/tenants/components/tenant-details-section";
import { TenantHeader } from "@/features/tenants/components/tenant-header";
import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";

const SKELETON = (
  <>
    <SectionSkeleton rows={2} />
    <SectionSkeleton rows={1} />
  </>
);

/**
 * As Configurações da comunidade: à esquerda, o que o dono decide sobre ela
 * (nome e descrição, endereço, logo, rating) e o apelido de quem vê; à
 * direita, os avisos no Discord (só o dono, a seção mais longa); por último,
 * a zona de perigo (`CommunitySettingsColumns`).
 */
export default async function TenantSettingsPage({
  params,
}: PageProps<"/[locale]/[tenant]/settings">) {
  const t = await getTranslations("Tenant");
  return (
    <>
      <Suspense fallback={<PageHeaderSkeleton label={t("settings.loading")} />}>
        {params.then(({ tenant }) => (
          <TenantHeader slug={tenant} title={t("tabs.settings")} settings />
        ))}
      </Suspense>
      <Suspense fallback={SKELETON}>
        {params.then(({ tenant }) => (
          <CommunitySettingsColumns
            slug={tenant}
            identity={
              <>
                <TenantDetailsSection slug={tenant} />
                <TenantAddressSection slug={tenant} />
                <Suspense fallback={<SectionSkeleton rows={2} />}>
                  <LogoSection slug={tenant} />
                </Suspense>
                <Suspense fallback={<SectionSkeleton rows={4} />}>
                  <RatingSettingsSection slug={tenant} />
                </Suspense>
                <NicknameSection slug={tenant} />
              </>
            }
            discord={
              <Suspense fallback={<SectionSkeleton rows={4} />}>
                <DiscordSection slug={tenant} />
              </Suspense>
            }
            danger={<CommunityDangerZone slug={tenant} />}
          />
        ))}
      </Suspense>
    </>
  );
}
