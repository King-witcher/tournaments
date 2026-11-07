import { apiErrorMessage } from "@/shared/api/errors";
import { Notice } from "@/shared/ui/notice";
import { soleProviderWithoutEmail } from "../lib/backup-login";
import { PROVIDERS } from "../lib/providers";
import { getEmailProof, getLogins } from "../server";
import { AccountColumns } from "./account-columns";
import { BackupLoginNotice } from "./backup-login-notice";
import { EmailLoginSection } from "./email-login-section";
import { PasswordLoginSection } from "./password-login-section";
import { ProviderLoginSection } from "./provider-login-section";

/**
 * Os Logins da conta, em duas metades (`AccountColumns`): à esquerda o email
 * e, logo embaixo, a senha, que dependem um do outro (o email é o login da
 * senha, ela só se cria com ele, e com ela ele não sai); à direita o Discord e
 * o Google, cada um para ligar ou remover. Email confirmado já entra na
 * conta, então cada login só sai se sobrar outro, e a senha sai sempre. Quem só
 * tem um provedor vê, em cima do email, o aviso de pôr um, que aqui não se
 * fecha. Na conta que só entra pelo email, mudar qualquer login pede antes o
 * código mandado a ele (`locked`). Fica em `<Suspense>`.
 */
export async function AccountLogins() {
  const result = await getLogins();
  if (!result) return null;
  if (!result.ok) {
    return <Notice kind="error">{await apiErrorMessage(result.code)}</Notice>;
  }
  const { logins } = result;
  const hasPassword = logins.password;
  const hasEmail = Boolean(logins.email);
  const providerLinked = PROVIDERS.some((provider) => logins[provider]);
  const sole = soleProviderWithoutEmail(logins);
  // Só nessa conta há o que perguntar: as outras não gastam a ida ao Redis.
  const locked =
    hasEmail &&
    !hasPassword &&
    !providerLinked &&
    !(await getEmailProof())?.proven;

  return (
    <AccountColumns
      start={
        <>
          {sole && <BackupLoginNotice provider={sole} />}
          <EmailLoginSection
            email={logins.email ?? undefined}
            hasPassword={hasPassword}
            providerLinked={providerLinked}
            locked={locked}
          />
          <PasswordLoginSection
            hasPassword={hasPassword}
            hasEmail={hasEmail}
            locked={locked}
          />
        </>
      }
      end={PROVIDERS.map((provider) => (
        <ProviderLoginSection
          key={provider}
          provider={provider}
          linked={logins[provider]}
          hasPassword={hasPassword}
          canUnlink={
            hasEmail ||
            PROVIDERS.some((other) => other !== provider && logins[other])
          }
          locked={locked}
        />
      ))}
    />
  );
}
