import { getTranslations } from "next-intl/server";
import { Suspense } from "react";
import { RatingSettingsSection } from "@/features/ratings/components/rating-settings-section";
import { TenantHeader } from "@/features/tenants/components/tenant-header";
import { Panel } from "@/shared/ui/panel";
import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";

/** Rating, nas Configurações da comunidade: só do owner. */
export default async function TenantRatingSettingsPage({
  params,
}: PageProps<"/[locale]/[tenant]/settings/rating">) {
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
            <SectionSkeleton rows={3} />
          </Panel>
        }
      >
        {params.then(({ tenant }) => (
          <RatingSettingsSection slug={tenant} />
        ))}
      </Suspense>
    </>
  );
}
