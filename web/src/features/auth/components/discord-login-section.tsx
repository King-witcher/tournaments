import { getTranslations } from "next-intl/server";

import { DiscordButton } from "./discord-button";
import { RemoveLoginForm } from "./remove-login-form";

type Props = { linked: boolean; hasPassword: boolean };

/** Discord da conta: vincular, ou desvincular se houver senha para entrar. */
export async function DiscordLoginSection({ linked, hasPassword }: Props) {
  const t = await getTranslations("Account.discord");

  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-xl font-semibold">{t("title")}</h2>
      {!linked ? (
        <>
          <p className="text-sm text-muted">{t("notLinked")}</p>
          <DiscordButton action="link" returnTo="/account">
            {t("link")}
          </DiscordButton>
        </>
      ) : (
        <>
          <p className="text-sm text-muted">{t("linked")}</p>
          {hasPassword ? (
            <>
              <p className="text-sm text-muted">{t("unlinkHint")}</p>
              <RemoveLoginForm login="discord" />
            </>
          ) : (
            <p className="text-sm text-muted">{t("needsPassword")}</p>
          )}
        </>
      )}
    </section>
  );
}
