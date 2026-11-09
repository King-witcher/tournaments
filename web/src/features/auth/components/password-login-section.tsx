import { getTranslations } from "next-intl/server";
import { Notice } from "@/shared/ui/notice";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import { LOGIN_SECTION } from "../lib/login-sections";
import { ChangePasswordForm } from "./change-password-form";
import { CredentialsForm } from "./credentials-form";

type Props = {
  hasPassword: boolean;
  hasEmail: boolean;
  /** A conta só entra pelo email, e esta sessão não digitou o código dele. */
  locked: boolean;
};

/**
 * A senha da conta, que entra com ela e o email: criar, se houver email, ou
 * trocar e remover (o email fica, e a conta continua entrando por ele). Sem
 * email ou com `locked`, o motivo no lugar do formulário. O bloco recebe o foco
 * quando o formulário some (`tabIndex={-1}`).
 */
export async function PasswordLoginSection({
  hasPassword,
  hasEmail,
  locked,
}: Props) {
  const t = await getTranslations("Account.password");
  const tEmail = await getTranslations("Account.email");

  return (
    <div id={LOGIN_SECTION.password} tabIndex={-1}>
      <Panel>
        <Section title={t("title")}>
          {locked ? (
            <Notice kind="info">{tEmail("proofFirst")}</Notice>
          ) : hasPassword ? (
            <ChangePasswordForm />
          ) : hasEmail ? (
            <CredentialsForm />
          ) : (
            <Notice kind="info">{t("needsEmail")}</Notice>
          )}
        </Section>
      </Panel>
    </div>
  );
}
