import { getTranslations } from "next-intl/server";
import { Suspense } from "react";
import { DiscordSection } from "@/features/discord/components/discord-section";
import { TenantHeader } from "@/features/tenants/components/tenant-header";
import { Panel } from "@/shared/ui/panel";
import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";

/** Discord, nas Configurações da comunidade: só do owner. */
export default async function TenantDiscordSettingsPage({
  params,
}: PageProps<"/[locale]/[tenant]/settings/discord">) {
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
      <Suspense
        fallback={
          <Panel>
            <SectionSkeleton rows={4} />
          </Panel>
        }
      >
        {params.then(({ tenant }) => (
          <DiscordSection slug={tenant} />
        ))}
      </Suspense>
    </>
  );
}
