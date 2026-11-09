import { getTranslations } from "next-intl/server";
import { Panel } from "@/shared/ui/panel";

import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";

/** Rating, nas Configurações, carregando. */
export default async function Loading() {
  const t = await getTranslations("Tenant.settings");
  return (
    <>
      <PageHeaderSkeleton breadcrumb="belowLg" label={t("loading")} />
      <Panel>
        <SectionSkeleton rows={3} />
      </Panel>
    </>
  );
}
