import { getTranslations } from "next-intl/server";

import { Notice } from "@/shared/ui/notice";

type Props = {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
};

/**
 * Na aba Senha, para quem acabou de trocar a senha pelo link do email
 * (`?password_reset=1`, de `PasswordResetForm`): diz que trocou e que as
 * outras sessões saíram. Fica em `<Suspense>`.
 */
export async function PasswordResetDone({ searchParams }: Props) {
  const { password_reset } = await searchParams;
  if (password_reset === undefined) return null;
  const t = await getTranslations("Account.password");

  return (
    <Notice kind="success" title={t("resetDoneTitle")}>
      {t("resetDone")}
    </Notice>
  );
}
