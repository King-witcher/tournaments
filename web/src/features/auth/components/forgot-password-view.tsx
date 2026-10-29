import { Layer, Stack, Tile } from "@carbon/react";
import { getTranslations } from "next-intl/server";

import { Notice } from "@/shared/ui/notice";
import { AuthSwitch } from "./auth-switch";
import { ForgotPasswordForm } from "./forgot-password-form";
import { OAuthButtons } from "./oauth-buttons";
import { OrDivider } from "./or-divider";

/**
 * Esqueci a senha: no `Tile`, o pedido do link e, como a outra saída, o
 * aviso de que conta sem email não se recupera por email e os provedores
 * (quem vinculou o Discord ou o Google entra por ele); o link de volta ao
 * login embaixo.
 */
export async function ForgotPasswordView() {
  const t = await getTranslations("Login.forgot");

  return (
    <Stack gap={6}>
      <Tile>
        <Layer>
          <Stack gap={6}>
            <ForgotPasswordForm />
            <OrDivider />
            <Notice kind="info">{t("noEmail")}</Notice>
            <OAuthButtons returnTo="/" />
          </Stack>
        </Layer>
      </Tile>
      <AuthSwitch
        text={t("remembered")}
        label={t("backToLogin")}
        href="/login"
      />
    </Stack>
  );
}
