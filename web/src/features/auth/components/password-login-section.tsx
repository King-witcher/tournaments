import { getTranslations } from "next-intl/server";

import { ChangePasswordForm } from "./change-password-form";
import { CredentialsForm } from "./credentials-form";
import { RemoveLoginForm } from "./remove-login-form";

type Props = { username?: string; discordLinked: boolean };

/** Nome de usuário e senha da conta: criar, ou trocar e remover. */
export async function PasswordLoginSection({ username, discordLinked }: Props) {
  const t = await getTranslations("Account.password");

  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-xl font-semibold">{t("title")}</h2>
      {!username ? (
        <>
          <p className="text-sm opacity-70">{t("none")}</p>
          <CredentialsForm mode="add" />
        </>
      ) : (
        <>
          <p className="text-sm">{t("current", { username })}</p>
          <ChangePasswordForm username={username} />
          <h3 className="pt-2 font-semibold">{t("remove")}</h3>
          {discordLinked ? (
            <>
              <p className="text-sm opacity-70">{t("removeHint")}</p>
              <RemoveLoginForm login="password" />
            </>
          ) : (
            <p className="text-sm opacity-70">{t("needsDiscord")}</p>
          )}
        </>
      )}
    </section>
  );
}
