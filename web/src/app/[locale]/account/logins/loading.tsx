import { getTranslations } from "next-intl/server";

import { LoginsSkeleton } from "@/features/auth/components/logins-skeleton";
import { PageHeaderSkeleton } from "@/shared/ui/skeleton";

/** Os Logins carregando: o cabeçalho e as duas metades. */
export default async function Loading() {
  const t = await getTranslations("Account");
  return (
    <>
      <PageHeaderSkeleton breadcrumb={false} label={t("loading")} />
      <LoginsSkeleton label={false} />
    </>
  );
}
