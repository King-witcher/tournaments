import { getTranslations } from "next-intl/server";

import { LoginsSkeleton } from "@/features/auth/components/logins-skeleton";
import { TabLoading } from "@/shared/ui/skeleton";

/** A aba Logins carregando: os blocos do email, dos provedores e da senha. */
export default async function Loading() {
  const t = await getTranslations("Account");
  return (
    <TabLoading label={t("loading")}>
      <LoginsSkeleton label={false} />
    </TabLoading>
  );
}
