import { Layer, Stack, Tile } from "@carbon/react";
import { getTranslations } from "next-intl/server";

import { Notice } from "@/shared/ui/notice";
import { safeReturnTo } from "../lib/return-to";
import { AlreadySignedIn } from "./already-signed-in";
import { AuthSwitch } from "./auth-switch";
import { CredentialsForm } from "./credentials-form";
import { DiscordButton } from "./discord-button";
import { OrDivider } from "./or-divider";

type Props = {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
};

/**
 * Criar conta com senha (ou entrar com Discord, que cria sozinho), voltando
 * para `?return_to=`: no `Tile`, o aviso de que não há recuperação por email
 * antes dos campos, o formulário e o Discord; o link para entrar embaixo.
 * Fica em `<Suspense>`.
 */
export async function SignupView({ searchParams }: Props) {
  const [t, tAuth, params] = await Promise.all([
    getTranslations("Signup"),
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
            <Notice kind="info">{t("noRecovery")}</Notice>
            <CredentialsForm mode="signup" returnTo={returnTo} />
            <OrDivider />
            <DiscordButton action="login" returnTo={returnTo}>
              {tAuth("discord")}
            </DiscordButton>
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
