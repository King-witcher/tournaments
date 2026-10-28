import { getTranslations } from "next-intl/server";

import { AccountTab } from "@/features/auth/components/account-tab";
import { LoginsSkeleton } from "@/features/auth/components/logins-skeleton";

/** A aba Logins carregando: o bloco do Discord e o da senha. */
export default async function Loading() {
  const t = await getTranslations("Account");
  return (
    <AccountTab>
      <LoginsSkeleton label={t("loading")} />
    </AccountTab>
  );
}
