import { getTranslations } from "next-intl/server";

/**
 * Traduz um código de erro da API. O código vira o caminho da chave em
 * `ApiErrors` ("auth.state_mismatch" → ApiErrors.auth.state_mismatch);
 * código desconhecido cai em "internal".
 */
export async function apiErrorMessage(code: string) {
  const t = await getTranslations("ApiErrors");
  type Key = Parameters<typeof t>[0];
  const key = code as Key;
  return t.has(key) ? t(key) : t("internal");
}
