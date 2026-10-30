"use client";

import { useTranslations } from "next-intl";

import { ErrorPage } from "@/features/errors/components/error-page";
import { digestOf } from "@/features/errors/lib/digest";

/**
 * Erro de execução em qualquer página do idioma (a API fora do ar, um erro de
 * renderização): dentro do layout, com o cabeçalho, sem a mensagem do erro.
 */
export default function LocaleError({
  error,
  retry,
}: {
  error: unknown;
  retry: () => void;
}) {
  const t = useTranslations("Errors");
  return (
    <ErrorPage title={t("title")} retry={retry} digest={digestOf(error)}>
      {t("text")}
    </ErrorPage>
  );
}
