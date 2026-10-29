import { Layer, Stack, Tile } from "@carbon/react";
import { getTranslations } from "next-intl/server";

import { safeReturnTo } from "../lib/return-to";
import { AccountDeletionScheduled } from "./account-deletion-scheduled";
import { AlreadySignedIn } from "./already-signed-in";
import { AuthSwitch } from "./auth-switch";
import { LoginForm } from "./login-form";
import { OAuthButtons } from "./oauth-buttons";
import { OrDivider } from "./or-divider";

type Props = {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
};

/**
 * Entrar com senha, com o Discord ou com o Google, voltando para
 * `?return_to=`: o formulário e os provedores num `Tile` (o `Layer` dá aos
 * campos a cor de campo sobre o tile), e o link para criar conta embaixo.
 * Logo depois de pedir a exclusão da conta (`?deletion_scheduled=`), a data
 * e como cancelar em cima. Fica em `<Suspense>`.
 */
export async function LoginView({ searchParams }: Props) {
  const [t, params] = await Promise.all([
    getTranslations("Login"),
    searchParams,
  ]);
  const returnTo = safeReturnTo(params.return_to);

  return (
    <Stack gap={6}>
      <AccountDeletionScheduled value={params.deletion_scheduled} />
      <AlreadySignedIn returnTo={returnTo} />
      <Tile>
        <Layer>
          <Stack gap={6}>
            <LoginForm returnTo={returnTo} />
            <OrDivider />
            <OAuthButtons returnTo={returnTo} />
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
