import { getTranslations } from "next-intl/server";

import { Notice } from "@/shared/ui/notice";

type Props = {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
};

/**
 * Na aba Perfil, para quem entrou numa conta com a exclusão agendada
 * (`?account_deletion=cancelled`, do login com senha ou com o Discord): diz
 * que a exclusão foi cancelada. Fica em `<Suspense>`.
 */
export async function AccountDeletionCancelled({ searchParams }: Props) {
  const { account_deletion } = await searchParams;
  if (account_deletion !== "cancelled") return null;
  const t = await getTranslations("Account.deletion");

  return (
    <Notice kind="success" title={t("cancelledTitle")}>
      {t("cancelled")}
    </Notice>
  );
}
