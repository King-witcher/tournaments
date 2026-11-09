import { getTranslations } from "next-intl/server";
import { Stack } from "@/shared/ui/carbon/stack";
import { Panel } from "@/shared/ui/panel";

import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";

/** Geral, nas Configurações, carregando: o cabeçalho e as primeiras seções. */
export default async function Loading() {
  const t = await getTranslations("Tenant.settings");
  return (
    <>
      <PageHeaderSkeleton breadcrumb="belowLg" label={t("loading")} />
      <Stack gap={7}>
        <Panel>
          <SectionSkeleton rows={3} />
        </Panel>
        <Panel>
          <SectionSkeleton rows={1} />
        </Panel>
      </Stack>
    </>
  );
}
