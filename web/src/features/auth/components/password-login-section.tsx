import { getTranslations } from "next-intl/server";

import { Notice } from "@/shared/ui/notice";
import { Section } from "@/shared/ui/section";
import { LOGIN_SECTION } from "../lib/login-sections";
import { CredentialsForm } from "./credentials-form";
import { RemoveLoginForm } from "./remove-login-form";

type Props = { username?: string; discordLinked: boolean; hasEmail: boolean };

/**
 * Nome de usuário e senha da conta: criar, se houver email (que recupera a
 * senha esquecida), ou remover, se o Discord estiver vinculado (trocar fica
 * na aba Senha). Sem um dos dois, o motivo no lugar do formulário. O bloco
 * recebe o foco quando o formulário some (`tabIndex={-1}`).
 */
export async function PasswordLoginSection({
  username,
  discordLinked,
  hasEmail,
}: Props) {
  const t = await getTranslations("Account.password");

  return (
    <div id={LOGIN_SECTION.password} tabIndex={-1}>
      <Section
        title={t("title")}
        description={username ? t("current", { username }) : t("none")}
      >
        {!username ? (
          hasEmail ? (
            <CredentialsForm />
          ) : (
            <Notice kind="info">{t("needsEmail")}</Notice>
          )
        ) : discordLinked ? (
          <RemoveLoginForm login="password" />
        ) : (
          <Notice kind="info">{t("needsDiscord")}</Notice>
        )}
      </Section>
    </div>
  );
}
