import { Layer, Stack, Tile } from "@carbon/react";
import { getTranslations } from "next-intl/server";

import { safeReturnTo } from "../lib/return-to";
import { AlreadySignedIn } from "./already-signed-in";
import { AuthSwitch } from "./auth-switch";
import { OAuthButtons } from "./oauth-buttons";
import { OrDivider } from "./or-divider";
import { SignupForm } from "./signup-form";

type Props = {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
};

/**
 * Criar conta com senha (ou entrar com Discord ou Google, que criam
 * sozinhos), voltando para `?return_to=`: no `Tile`, o formulário (que,
 * enviado, vira o "confira seu email") e os provedores; o link para entrar
 * embaixo. Fica em `<Suspense>`.
 */
export async function SignupView({ searchParams }: Props) {
  const [t, params] = await Promise.all([
    getTranslations("Signup"),
    searchParams,
  ]);
  const returnTo = safeReturnTo(params.return_to);

  return (
    <Stack gap={6}>
      <AlreadySignedIn returnTo={returnTo} />
      <Tile>
        <Layer>
          <Stack gap={6}>
            <SignupForm returnTo={returnTo} />
            <OrDivider />
            <OAuthButtons returnTo={returnTo} />
          </Stack>
        </Layer>
      </Tile>
      <AuthSwitch
        text={t("hasAccount")}
        label={t("login")}
        href={{ pathname: "/login", query: { return_to: returnTo } }}
      />
    </Stack>
  );
}
