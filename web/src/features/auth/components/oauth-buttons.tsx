import { Stack } from "@carbon/react";
import { getTranslations } from "next-intl/server";

import { OAuthButton } from "./oauth-button";

/**
 * Entrar com o Discord ou com o Google, um botão embaixo do outro, voltando
 * para `returnTo`. Mais perto um do outro que do formulário: são as duas
 * opções do "ou".
 */
export async function OAuthButtons({ returnTo }: { returnTo: string }) {
  const t = await getTranslations("Auth");
  return (
    <Stack gap={4}>
      <OAuthButton provider="discord" action="login" returnTo={returnTo}>
        {t("discord")}
      </OAuthButton>
      <OAuthButton provider="google" action="login" returnTo={returnTo}>
        {t("google")}
      </OAuthButton>
    </Stack>
  );
}
