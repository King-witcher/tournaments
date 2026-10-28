import { apiErrorMessage } from "@/shared/api/errors";
import { Notice } from "@/shared/ui/notice";
import { getLogins } from "../server";
import { DiscordLoginSection } from "./discord-login-section";
import { EmailLoginSection } from "./email-login-section";
import { PasswordLoginSection } from "./password-login-section";

/**
 * A aba Logins da conta: o email (primeiro, porque recupera a senha e o
 * desvincular do Discord depende dele; a conta sem email vê o aviso logo no
 * topo), o Discord e o nome de usuário com senha, cada um para ligar ou
 * remover. Fica em `<Suspense>`.
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
      <DiscordLoginSection
        linked={logins.discord}
        hasPassword={hasPassword}
        hasEmail={Boolean(logins.email)}
      />
      <PasswordLoginSection
        username={logins.username ?? undefined}
        discordLinked={logins.discord}
      />
    </>
  );
}
