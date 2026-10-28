import { getTranslations } from "next-intl/server";

import { FormSkeleton, TabLoading } from "@/shared/ui/skeleton";

/** A aba Perfil carregando, dentro das abas da conta: o título e o campo do nome. */
export default async function Loading() {
  const t = await getTranslations("Account");
  return (
    <TabLoading label={t("loading")}>
      <FormSkeleton heading fields={1} label={false} />
    </TabLoading>
  );
}
