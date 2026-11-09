import { getTranslations } from "next-intl/server";

import { SessionsSkeleton } from "@/features/auth/components/sessions-skeleton";
import { PageHeaderSkeleton } from "@/shared/ui/skeleton";

/** As Sessões carregando: o cabeçalho e a lista. */
export default async function Loading() {
  const t = await getTranslations("Account");
  return (
    <>
      <PageHeaderSkeleton breadcrumb={false} label={t("loading")} />
      <SessionsSkeleton label={false} />
    </>
  );
}
