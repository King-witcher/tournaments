"use client";

import { useTranslations } from "next-intl";

import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import type { ImageOutcome } from "../lib/upload-image";

type Failure = Exclude<ImageOutcome, { ok: true }>;

/** O texto de um envio de imagem que falhou, do navegador ou da API. */
export function useImageErrorMessage() {
  const t = useTranslations("Images.errors");
  const errorMessage = useApiErrorMessage();
  return (failure: Failure) =>
    "local" in failure
      ? t(failure.local)
      : errorMessage(failure.code, failure.params);
}
