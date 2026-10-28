import { getTranslations } from "next-intl/server";

import { NewTenantFormSkeleton } from "@/features/tenants/components/new-tenant-form-skeleton";
import { Page } from "@/shared/ui/page";
import { PageHeaderSkeleton } from "@/shared/ui/skeleton";

/** "Criar comunidade" carregando: o título e os dois campos. */
export default async function Loading() {
  const t = await getTranslations("NewTenant");
  return (
    <Page>
      <PageHeaderSkeleton meta label={t("loading")} />
      <NewTenantFormSkeleton />
    </Page>
  );
}
