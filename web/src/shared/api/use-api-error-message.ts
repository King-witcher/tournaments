import { useTranslations } from "next-intl";

import type { ApiErrorParams } from "./call";

/**
 * Versão de `apiErrorMessage` (errors.ts) para Client Components: devolve uma
 * função que traduz o código de erro da API, interpolando os números do erro;
 * código ausente ou desconhecido cai em "internal".
 */
export function useApiErrorMessage() {
  const t = useTranslations("ApiErrors");
  type Key = Parameters<typeof t>[0];
  type Values = Parameters<typeof t>[1];
  return (code: string | undefined, params?: ApiErrorParams | null) => {
    const key = (code ?? "internal") as Key;
    return t.has(key) ? t(key, (params ?? undefined) as Values) : t("internal");
  };
}
