import { getTranslations } from "next-intl/server";

import { AccountTab } from "@/features/auth/components/account-tab";
import { FormSkeleton } from "@/features/auth/components/form-skeleton";

/** A aba Perfil carregando, dentro das abas da conta: o título e o campo do nome. */
export default async function Loading() {
  const t = await getTranslations("Account");
  return (
    <AccountTab>
      <FormSkeleton heading fields={1} label={t("loading")} />
    </AccountTab>
  );
}
