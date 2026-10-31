import { apiErrorMessage } from "@/shared/api/errors";
import { Notice } from "@/shared/ui/notice";
import { PROVIDERS } from "../lib/providers";
import { getLogins } from "../server";
import { EmailLoginSection } from "./email-login-section";
import { PasswordLoginSection } from "./password-login-section";
import { ProviderLoginSection } from "./provider-login-section";

/**
 * A aba Logins da conta: o email (primeiro, porque a senha depende dele: é o
 * login dela, só se cria com email, e com senha ele não sai), o Discord, o
 * Google e a senha, cada um para ligar ou remover. A conta nunca fica sem
 * jeito de entrar: cada um só sai se sobrar outro. Fica em `<Suspense>`.
 */
export async function AccountLogins() {
  const result = await getLogins();
  if (!result) return null;
  if (!result.ok) {
    return <Notice kind="error">{await apiErrorMessage(result.code)}</Notice>;
  }
  const { logins } = result;
  const hasPassword = logins.password;

  return (
    <>
      <EmailLoginSection
        email={logins.email ?? undefined}
        hasPassword={hasPassword}
      />
      {PROVIDERS.map((provider) => (
        <ProviderLoginSection
          key={provider}
          provider={provider}
          linked={logins[provider]}
          hasPassword={hasPassword}
          canUnlink={
            hasPassword ||
            PROVIDERS.some((other) => other !== provider && logins[other])
          }
        />
      ))}
      <PasswordLoginSection
        hasPassword={hasPassword}
        providerLinked={PROVIDERS.some((provider) => logins[provider])}
        hasEmail={Boolean(logins.email)}
      />
    </>
  );
}
