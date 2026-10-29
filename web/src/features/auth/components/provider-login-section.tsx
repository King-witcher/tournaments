import { getTranslations } from "next-intl/server";

import { Notice } from "@/shared/ui/notice";
import { Section } from "@/shared/ui/section";
import { LOGIN_SECTION } from "../lib/login-sections";
import type { Provider } from "../lib/providers";
import { LinkProviderForm } from "./link-provider-form";
import { OAuthButton } from "./oauth-button";
import { RemoveLoginForm } from "./remove-login-form";

type Props = {
  provider: Provider;
  linked: boolean;
  /** A conta tem senha: vincular e desvincular pedem a atual. */
  hasPassword: boolean;
  /** Sobra outro jeito de entrar (a senha ou o outro provedor). */
  canUnlink: boolean;
};

/**
 * Um provedor da conta (Discord ou Google): vincular (com a senha atual, se
 * a conta tem), ou desvincular se sobrar outro jeito de entrar. Sem outro, o
 * motivo no lugar do formulário.
 * O bloco recebe o foco quando o formulário some (`tabIndex={-1}`).
 */
export async function ProviderLoginSection({
  provider,
  linked,
  hasPassword,
  canUnlink,
}: Props) {
  const t = await getTranslations("Account");

  return (
    <div id={LOGIN_SECTION[provider]} tabIndex={-1}>
      <Section
        title={t(`${provider}.title`)}
        description={
          linked ? t(`${provider}.linked`) : t(`${provider}.notLinked`)
        }
      >
        {!linked && hasPassword ? (
          <LinkProviderForm provider={provider} returnTo="/account/logins" />
        ) : !linked ? (
          <OAuthButton
            provider={provider}
            action="link"
            returnTo="/account/logins"
          >
            {t(`${provider}.link`)}
          </OAuthButton>
        ) : !canUnlink ? (
          <Notice kind="info">{t(`${provider}.needsOther`)}</Notice>
        ) : (
          <RemoveLoginForm login={provider} needsPassword={hasPassword} />
        )}
      </Section>
    </div>
  );
}
