import { getTranslations } from "next-intl/server";

import { SessionsSkeleton } from "@/features/auth/components/sessions-skeleton";
import { TabLoading } from "@/shared/ui/skeleton";

/** A aba Sessões carregando. */
export default async function Loading() {
  const t = await getTranslations("Account");
  return (
    <TabLoading label={t("loading")}>
      <SessionsSkeleton label={false} />
    </TabLoading>
  );
}
