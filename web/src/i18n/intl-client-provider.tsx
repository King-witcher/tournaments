"use client";

import {
  type IntlConfig,
  type IntlError,
  IntlErrorCode,
  NextIntlClientProvider,
} from "next-intl";
import { type ReactNode, useState } from "react";

type Props = {
  locale: IntlConfig["locale"];
  timeZone: IntlConfig["timeZone"];
  /** As do `clientMessages` (`namespaces.ts`). */
  messages: NonNullable<IntlConfig["messages"]>;
  children: ReactNode;
};

/**
 * O provider do next-intl no navegador, no layout raiz. Recebe do servidor o
 * que o `NextIntlClientProvider` de servidor leria do `request.ts` (o idioma,
 * o fuso e as mensagens); se o `request.ts` passar a definir `formats` ou
 * `now`, eles entram aqui também.
 *
 * Cada `router.refresh()` (`LiveUpdates`, ações) refaz o layout no servidor, e
 * as mensagens chegam iguais, mas num objeto novo: o contexto do next-intl
 * mudaria e todo componente com `useTranslations` renderizaria de novo. Em
 * produção elas só mudam com o idioma (vêm do build), então o objeto do
 * primeiro render continua valendo; em dev, mudam ao editar o JSON, e vale o
 * que chegou.
 */
export function IntlClientProvider({
  locale,
  timeZone,
  messages,
  children,
}: Props) {
  const [kept, setKept] = useState({ locale, messages });
  if (kept.locale !== locale) setKept({ locale, messages });
  const stable =
    process.env.NODE_ENV === "production" && kept.locale === locale
      ? kept.messages
      : messages;
  return (
    <NextIntlClientProvider
      locale={locale}
      timeZone={timeZone}
      messages={stable}
      onError={onError}
    >
      {children}
    </NextIntlClientProvider>
  );
}

/**
 * O padrão do next-intl (o erro no console) e, em dev, o motivo provável da
 * mensagem que falta: as chaves são conferidas pelo `tsc`, então a que existe
 * no JSON e não chegou aqui está em `SERVER_ONLY_MESSAGES`.
 */
function onError(error: IntlError) {
  console.error(error);
  if (
    process.env.NODE_ENV !== "production" &&
    error.code === IntlErrorCode.MISSING_MESSAGE
  ) {
    console.error(
      "Mensagem ausente num componente de cliente: se ela existe em messages/, tire o namespace (ou a parte) de SERVER_ONLY_MESSAGES em src/i18n/namespaces.ts.",
    );
  }
}
