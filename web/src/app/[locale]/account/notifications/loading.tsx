import { getTranslations } from "next-intl/server";

import { AccountColumns } from "@/features/auth/components/account-columns";
import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";

/** As Notificações carregando: o cabeçalho e as duas seções lado a lado. */
export default async function Loading() {
  const t = await getTranslations("Account");
  return (
    <>
      <PageHeaderSkeleton label={t("loading")} />
      <AccountColumns
        start={<SectionSkeleton rows={2} />}
        end={<SectionSkeleton rows={2} />}
      />
    </>
  );
}
