import { getTranslations } from "next-intl/server";

import { OAuthButton } from "./oauth-button";
import styles from "./oauth-buttons.module.scss";

/**
 * Entrar com o Discord ou com o Google, um botão ao lado do outro (um embaixo
 * do outro quando não cabem), voltando para `returnTo`. Mais perto um do
 * outro que do formulário: são as duas opções do "ou".
 */
export async function OAuthButtons({ returnTo }: { returnTo: string }) {
  const t = await getTranslations("Auth");
  return (
    <div className={styles.buttons}>
      <OAuthButton provider="discord" action="login" returnTo={returnTo}>
        {t("discord")}
      </OAuthButton>
      <OAuthButton provider="google" action="login" returnTo={returnTo}>
        {t("google")}
      </OAuthButton>
    </div>
  );
}
