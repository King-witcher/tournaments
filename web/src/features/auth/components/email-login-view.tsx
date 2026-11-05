import { getTranslations } from "next-intl/server";

import { Layer } from "@/shared/ui/carbon/layer";
import { Stack } from "@/shared/ui/carbon/stack";
import { Tile } from "@/shared/ui/carbon/tile";
import { Notice } from "@/shared/ui/notice";
import { safeReturnTo } from "../lib/return-to";
import { AlreadySignedIn } from "./already-signed-in";
import { AuthSwitch } from "./auth-switch";
import { EmailLogin } from "./email-login";
import { OAuthButtons } from "./oauth-buttons";
import { OrDivider } from "./or-divider";

type Props = {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
};

/**
 * Entrar por email: o pedido no `Tile` e, como a outra saída, os provedores; o
 * link de volta ao login embaixo. O campo vem com o `?email=` que o login levou.
 * Quem chega por um link antigo de recuperação de senha (`?expired=1`, do
 * `next.config.ts`) lê que ele expirou. Fica em `<Suspense>`.
 */
export async function EmailLoginView({ searchParams }: Props) {
  const [t, params] = await Promise.all([
    getTranslations("Login.email"),
    searchParams,
  ]);
  const email = typeof params.email === "string" ? params.email : "";
  const returnTo = safeReturnTo(params.return_to);

  return (
    <Stack gap={6}>
      {params.expired !== undefined && (
        <Notice kind="info">{t("expired")}</Notice>
      )}
      <AlreadySignedIn returnTo={returnTo} />
      <Tile>
        <Layer>
          <EmailLogin
            initialEmail={email.slice(0, 254)}
            returnTo={returnTo}
            alternatives={
              <>
                <OrDivider />
                <OAuthButtons returnTo={returnTo} />
              </>
            }
          />
        </Layer>
      </Tile>
      <AuthSwitch
        text={t("remembered")}
        label={t("backToLogin")}
        href={{ pathname: "/login", query: { return_to: returnTo } }}
      />
    </Stack>
  );
}
