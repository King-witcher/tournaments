import { Layer, Stack, Tile } from "@carbon/react";
import { getTranslations } from "next-intl/server";

import { safeReturnTo } from "../lib/return-to";
import { AlreadySignedIn } from "./already-signed-in";
import { AuthSwitch } from "./auth-switch";
import { DiscordButton } from "./discord-button";
import { LoginForm } from "./login-form";
import { OrDivider } from "./or-divider";

type Props = {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
};

/**
 * Entrar com senha ou com o Discord, voltando para `?return_to=`: o
 * formulário e o Discord num `Tile` (o `Layer` dá aos campos a cor de campo
 * sobre o tile), e o link para criar conta embaixo. Fica em `<Suspense>`.
 */
export async function LoginView({ searchParams }: Props) {
  const [t, tAuth, params] = await Promise.all([
    getTranslations("Login"),
    getTranslations("Auth"),
    searchParams,
  ]);
  const returnTo = safeReturnTo(params.return_to);

  return (
    <Stack gap={6}>
      <AlreadySignedIn returnTo={returnTo} />
      <Tile>
        <Layer>
          <Stack gap={6}>
            <LoginForm returnTo={returnTo} />
            <OrDivider />
            <DiscordButton action="login" returnTo={returnTo}>
              {tAuth("discord")}
            </DiscordButton>
          </Stack>
        </Layer>
      </Tile>
      <AuthSwitch
        text={t("noAccount")}
        label={t("signup")}
        href={{ pathname: "/signup", query: { return_to: returnTo } }}
      />
    </Stack>
  );
}
