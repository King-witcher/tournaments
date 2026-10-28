import { getTranslations } from "next-intl/server";

import { Notice } from "@/shared/ui/notice";
import { Section } from "@/shared/ui/section";
import { LOGIN_SECTION } from "../lib/login-sections";
import { DiscordButton } from "./discord-button";
import { RemoveLoginForm } from "./remove-login-form";

type Props = { linked: boolean; hasPassword: boolean };

/**
 * Discord da conta: vincular, ou desvincular se houver senha para entrar
 * (que sempre vem com o email que a recupera). Sem senha, o motivo no lugar
 * do formulário. O bloco recebe o foco quando o formulário some
 * (`tabIndex={-1}`).
 */
export async function DiscordLoginSection({ linked, hasPassword }: Props) {
  const t = await getTranslations("Account.discord");

  return (
    <div id={LOGIN_SECTION.discord} tabIndex={-1}>
      <Section
        title={t("title")}
        description={linked ? t("linked") : t("notLinked")}
      >
        {!linked ? (
          <DiscordButton action="link" returnTo="/account/logins">
            {t("link")}
          </DiscordButton>
        ) : !hasPassword ? (
          <Notice kind="info">{t("needsPassword")}</Notice>
        ) : (
          <RemoveLoginForm login="discord" />
        )}
      </Section>
    </div>
  );
}
