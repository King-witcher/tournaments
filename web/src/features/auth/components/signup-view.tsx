import { getTranslations } from "next-intl/server";

import { Link } from "@/i18n/navigation";
import { safeReturnTo } from "../lib/return-to";
import { AlreadySignedIn } from "./already-signed-in";
import { CredentialsForm } from "./credentials-form";
import { DiscordButton } from "./discord-button";

type Props = {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
};

/** Criar conta com senha (ou entrar com Discord, que cria sozinho). Fica em `<Suspense>`. */
export async function SignupView({ searchParams }: Props) {
  const [t, tAuth, params] = await Promise.all([
    getTranslations("Signup"),
    getTranslations("Auth"),
    searchParams,
  ]);
  const returnTo = safeReturnTo(params.return_to);

  return (
    <div className="flex flex-col gap-6">
      <AlreadySignedIn returnTo={returnTo} />
      <p className="text-sm text-muted">{t("noRecovery")}</p>
      <CredentialsForm mode="signup" returnTo={returnTo} />
      <p className="text-sm text-muted">{tAuth("or")}</p>
      <DiscordButton action="login" returnTo={returnTo}>
        {tAuth("discord")}
      </DiscordButton>
      <p className="text-sm">
        {t("hasAccount")}{" "}
        <Link
          href={{ pathname: "/login", query: { return_to: returnTo } }}
          className="underline"
        >
          {t("login")}
        </Link>
      </p>
    </div>
  );
}
