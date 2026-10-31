import { getTranslations } from "next-intl/server";

import { Notice } from "@/shared/ui/notice";
import { Section } from "@/shared/ui/section";
import { LOGIN_SECTION } from "../lib/login-sections";
import { ChangePasswordForm } from "./change-password-form";
import { CredentialsForm } from "./credentials-form";

type Props = {
  hasPassword: boolean;
  /** O Discord ou o Google está vinculado: sem senha, a conta ainda entra. */
  providerLinked: boolean;
  hasEmail: boolean;
};

/**
 * A senha da conta, que entra com ela e o email: criar, se houver email, ou
 * trocar e remover (remover só com o Discord ou o Google vinculado). Sem
 * email, o motivo no lugar do formulário. O bloco recebe o foco quando o
 * formulário some (`tabIndex={-1}`).
 */
export async function PasswordLoginSection({
  hasPassword,
  providerLinked,
  hasEmail,
}: Props) {
  const t = await getTranslations("Account.password");

  return (
    <div id={LOGIN_SECTION.password} tabIndex={-1}>
      <Section title={t("title")}>
        {hasPassword ? (
          <ChangePasswordForm canRemove={providerLinked} />
        ) : hasEmail ? (
          <CredentialsForm />
        ) : (
          <Notice kind="info">{t("needsEmail")}</Notice>
        )}
      </Section>
    </div>
  );
}
