import { apiErrorMessage } from "@/shared/api/errors";
import { Notice } from "@/shared/ui/notice";
import { getLogins } from "../server";
import { DiscordLoginSection } from "./discord-login-section";
import { EmailLoginSection } from "./email-login-section";
import { PasswordLoginSection } from "./password-login-section";

/**
 * A aba Logins da conta: o email (primeiro, porque a senha depende dele: só
 * se cria com email, e com senha ele não sai), o Discord e o nome de usuário
 * com senha, cada um para ligar ou remover. Fica em `<Suspense>`.
 */
export async function AccountLogins() {
  const result = await getLogins();
  if (!result) return null;
  if (!result.ok) {
    return <Notice kind="error">{await apiErrorMessage(result.code)}</Notice>;
  }
  const { logins } = result;
  const hasPassword = Boolean(logins.username);

  return (
    <>
      <EmailLoginSection
        email={logins.email ?? undefined}
        hasPassword={hasPassword}
      />
      <DiscordLoginSection linked={logins.discord} hasPassword={hasPassword} />
      <PasswordLoginSection
        username={logins.username ?? undefined}
        discordLinked={logins.discord}
        hasEmail={Boolean(logins.email)}
      />
    </>
  );
}
