import { getTranslations } from "next-intl/server";

import { Link } from "@/i18n/navigation";
import { safeReturnTo } from "../lib/return-to";
import { AlreadySignedIn } from "./already-signed-in";
import { DiscordButton } from "./discord-button";
import { LoginForm } from "./login-form";

type Props = {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
};

/** Entrar com Discord ou com senha, voltando para `?return_to=`. Fica em `<Suspense>`. */
export async function LoginView({ searchParams }: Props) {
  const [t, tAuth, params] = await Promise.all([
    getTranslations("Login"),
    getTranslations("Auth"),
    searchParams,
  ]);
  const returnTo = safeReturnTo(params.return_to);

  return (
    <div className="flex flex-col gap-6">
      <AlreadySignedIn returnTo={returnTo} />
      <DiscordButton action="login" returnTo={returnTo}>
        {tAuth("discord")}
      </DiscordButton>
      <p className="text-sm text-muted">{tAuth("or")}</p>
      <LoginForm returnTo={returnTo} />
      <p className="text-sm">
        {t("noAccount")}{" "}
        <Link
          href={{ pathname: "/signup", query: { return_to: returnTo } }}
          className="underline"
        >
          {t("signup")}
        </Link>
      </p>
    </div>
  );
}
