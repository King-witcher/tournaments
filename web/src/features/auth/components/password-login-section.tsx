import { getTranslations } from "next-intl/server";

import { Notice } from "@/shared/ui/notice";
import { Section } from "@/shared/ui/section";
import { LOGIN_SECTION } from "../lib/login-sections";
import { CredentialsForm } from "./credentials-form";
import { RemoveLoginForm } from "./remove-login-form";

type Props = { username?: string; discordLinked: boolean };

/**
 * Nome de usuário e senha da conta: criar, ou remover se o Discord estiver
 * vinculado (trocar fica na aba Senha). O bloco recebe o foco quando o
 * formulário some (`tabIndex={-1}`).
 */
export async function PasswordLoginSection({ username, discordLinked }: Props) {
  const t = await getTranslations("Account.password");

  return (
    <div id={LOGIN_SECTION.password} tabIndex={-1}>
      <Section
        title={t("title")}
        description={username ? t("current", { username }) : t("none")}
      >
        {!username ? (
          <CredentialsForm mode="add" />
        ) : discordLinked ? (
          <RemoveLoginForm login="password" />
        ) : (
          <Notice kind="info">{t("needsDiscord")}</Notice>
        )}
      </Section>
    </div>
  );
}
