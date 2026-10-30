"use client";

import "./globals.scss";

import { useSyncExternalStore } from "react";

import { StandaloneErrorDocument } from "@/features/errors/components/standalone-error-document";
import { digestOf } from "@/features/errors/lib/digest";

const subscribe = () => () => {};

/** O idioma pelo primeiro segmento do endereço; sem ele (ou no servidor), português. */
function currentLocale(): "pt" | "es" {
  return window.location.pathname.split("/")[1] === "es" ? "es" : "pt";
}

/**
 * Erro no layout raiz (ou no que o `error.tsx` não alcança): substitui a
 * página inteira, sem o provider do next-intl, por isso o idioma sai do
 * endereço e os textos vêm direto dos arquivos de mensagens.
 */
export default function GlobalError({
  error,
  retry,
}: {
  error: unknown;
  retry: () => void;
}) {
  const locale = useSyncExternalStore<"pt" | "es">(
    subscribe,
    currentLocale,
    () => "pt",
  );
  return (
    <StandaloneErrorDocument
      locales={[locale]}
      kind="failed"
      retry={retry}
      digest={digestOf(error)}
    />
  );
}
