import { getTranslations } from "next-intl/server";

import { apiErrorMessage } from "@/shared/api/errors";
import { Notice } from "@/shared/ui/notice";

type Props = {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
};

/**
 * Mostra o erro com que o callback do login pelo provedor (`OAuthCallback`)
 * volta, em `?auth_error=`. O bloqueio do Discord leva o motivo junto: sem
 * ele, parece defeito do site.
 */
export async function AuthError({ searchParams }: Props) {
  const { auth_error } = await searchParams;
  if (typeof auth_error !== "string") return null;

  const message = await apiErrorMessage(auth_error);
  if (auth_error === "auth.discord_rate_limited") {
    const t = await getTranslations("Auth");
    return (
      <Notice kind="error" title={message}>
        {t("discordRateLimitedWhy")}
      </Notice>
    );
  }
  return <Notice kind="error">{message}</Notice>;
}
