import { getTranslations } from "next-intl/server";
import { Panel } from "@/shared/ui/panel";

import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";

/** As Configurações carregando: o cabeçalho e as primeiras seções. */
export default async function Loading() {
  const t = await getTranslations("Tenant.settings");
  return (
    <>
      <PageHeaderSkeleton breadcrumb="belowLg" label={t("loading")} />
      <Panel>
        <SectionSkeleton rows={2} />
      </Panel>
      <Panel>
        <SectionSkeleton rows={1} />
      </Panel>
    </>
  );
}
