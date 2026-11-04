import { locale as rootLocale } from "next/root-params";
import { hasLocale } from "next-intl";
import { getRequestConfig } from "next-intl/server";

import { loadMessages } from "./namespaces";
import { routing } from "./routing";

// `explicit`: o idioma pedido em `getTranslations({ locale })`, diferente do da
// página (a prévia do link de inscrição sai no idioma do torneio).
export default getRequestConfig(async ({ locale: explicit }) => {
  const requested = explicit ?? (await rootLocale());
  const locale = hasLocale(routing.locales, requested)
    ? requested
    : routing.defaultLocale;

  return { locale, messages: await loadMessages(locale) };
});
