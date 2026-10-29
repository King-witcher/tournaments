import { getTranslations } from "next-intl/server";

import { apiErrorMessage } from "@/shared/api/errors";
import { ButtonLink } from "@/shared/ui/button-link";
import { EmptyState } from "@/shared/ui/empty-state";
import { Notice } from "@/shared/ui/notice";
import { Section } from "@/shared/ui/section";
import { getLogins } from "../server";
import { ChangePasswordForm } from "./change-password-form";

/**
 * A aba Senha da conta: trocar o nome de usuário e a senha. Sem senha (só
 * Discord ou Google), diz que não há o que trocar e leva à aba Logins, onde
 * se cria. Fica em `<Suspense>`.
 */
export async function ChangePasswordSection() {
  const [result, t] = await Promise.all([
    getLogins(),
    getTranslations("Account.password"),
  ]);
  if (!result) return null;
  if (!result.ok) {
    return <Notice kind="error">{await apiErrorMessage(result.code)}</Notice>;
  }
  const { username } = result.logins;

  return (
    <Section
      title={t("changeTitle")}
      description={username ? t("current", { username }) : undefined}
    >
      {username ? (
        <ChangePasswordForm username={username} />
      ) : (
        <EmptyState
          action={
            <ButtonLink href="/account/logins" kind="tertiary">
              {t("goToLogins")}
            </ButtonLink>
          }
        >
          {t("noPassword")}
        </EmptyState>
      )}
    </Section>
  );
}
