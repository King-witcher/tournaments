import { getTranslations } from "next-intl/server";

import { FormSkeleton, TabLoading } from "@/shared/ui/skeleton";

/** A aba Senha carregando: o título, o nome de usuário e as senhas. */
export default async function Loading() {
  const t = await getTranslations("Account");
  return (
    <TabLoading label={t("loading")}>
      <FormSkeleton heading text fields={3} label={false} />
    </TabLoading>
  );
}
