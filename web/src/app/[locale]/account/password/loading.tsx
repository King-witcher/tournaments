import { getTranslations } from "next-intl/server";

import { AccountTab } from "@/features/auth/components/account-tab";
import { FormSkeleton } from "@/features/auth/components/form-skeleton";

/** A aba Senha carregando: o título, o nome de usuário e as senhas. */
export default async function Loading() {
  const t = await getTranslations("Account");
  return (
    <AccountTab>
      <FormSkeleton heading text fields={3} label={t("loading")} />
    </AccountTab>
  );
}
