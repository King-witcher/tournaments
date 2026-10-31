import { Layer, Stack, Tile } from "@carbon/react";
import { getTranslations } from "next-intl/server";

import { AuthSwitch } from "./auth-switch";
import { ForgotPasswordForm } from "./forgot-password-form";
import { OAuthButtons } from "./oauth-buttons";
import { OrDivider } from "./or-divider";

type Props = {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
};

/**
 * Esqueci a senha: no `Tile`, o pedido do link e, como a outra saída, os
 * provedores (toda conta com senha tem email, e quem vinculou o Discord ou o
 * Google entra por ele); o link de volta ao login embaixo. O campo vem com o
 * `?email=` que o login levou. Fica em `<Suspense>`.
 */
export async function ForgotPasswordView({ searchParams }: Props) {
  const [t, params] = await Promise.all([
    getTranslations("Login.forgot"),
    searchParams,
  ]);
  const email = typeof params.email === "string" ? params.email : "";

  return (
    <Stack gap={6}>
      <Tile>
        <Layer>
          <Stack gap={6}>
            <ForgotPasswordForm initialEmail={email.slice(0, 254)} />
            <OrDivider />
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
