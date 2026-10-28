import { getTranslations } from "next-intl/server";

import { apiErrorMessage } from "@/shared/api/errors";
import { PageError } from "@/shared/ui/page-error";

type Props = { tenant: string; code: string };

/**
 * Erro ao abrir um torneio (ou algo dentro dele), com o caminho de volta: a
 * comunidade, se ela ainda abre; senão, a home.
 */
export async function ErrorBack({ tenant, code }: Props) {
  const [t, message] = await Promise.all([
    getTranslations("Nav"),
    apiErrorMessage(code),
  ]);
  const outside = code.startsWith("tenant.") || code.startsWith("auth.");
  return outside ? (
    <PageError href="/" label={t("backHome")}>
      {message}
    </PageError>
  ) : (
    <PageError href={`/${tenant}`} label={t("backToCommunity")}>
      {message}
    </PageError>
  );
}
