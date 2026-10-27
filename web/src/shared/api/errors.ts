import { getTranslations } from "next-intl/server";

import type { ApiErrorParams } from "./call";

/**
 * Traduz um código de erro da API. O código vira o caminho da chave em
 * `ApiErrors` ("auth.state_mismatch" → ApiErrors.auth.state_mismatch), com os
 * números do erro interpolados; código desconhecido cai em "internal".
 */
export async function apiErrorMessage(
  code: string,
  params?: ApiErrorParams | null,
) {
  const t = await getTranslations("ApiErrors");
  type Key = Parameters<typeof t>[0];
  type Values = Parameters<typeof t>[1];
  const key = code as Key;
  return t.has(key) ? t(key, (params ?? undefined) as Values) : t("internal");
}
