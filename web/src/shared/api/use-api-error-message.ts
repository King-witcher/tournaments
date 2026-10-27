import { useTranslations } from "next-intl";

/**
 * Versão de `apiErrorMessage` (errors.ts) para Client Components: devolve uma
 * função que traduz o código de erro da API; código ausente ou desconhecido
 * cai em "internal".
 */
export function useApiErrorMessage() {
  const t = useTranslations("ApiErrors");
  type Key = Parameters<typeof t>[0];
  return (code: string | undefined) => {
    const key = (code ?? "internal") as Key;
    return t.has(key) ? t(key) : t("internal");
  };
}
