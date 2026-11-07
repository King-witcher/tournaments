import { getTranslations } from "next-intl/server";

import { AccountColumns } from "@/features/auth/components/account-columns";
import {
  FormSkeleton,
  PageHeaderSkeleton,
  SectionSkeleton,
} from "@/shared/ui/skeleton";

/** O Perfil carregando: o título, o nome à esquerda e a foto à direita. */
export default async function Loading() {
  const t = await getTranslations("Account");
  return (
    <>
      <PageHeaderSkeleton breadcrumb={false} label={t("loading")} />
      <AccountColumns
        start={<FormSkeleton heading fields={1} label={false} />}
        end={<SectionSkeleton rows={2} />}
      />
    </>
  );
}
