"use client";

import { useTranslations } from "next-intl";

import { Button } from "@/shared/ui/button";
import { Notice } from "@/shared/ui/notice";
import { useRetrySession } from "../hooks/use-retry-session";

/**
 * No lugar do que pede sessão, quando a API não respondeu a tempo ou falhou:
 * a pessoa não saiu da conta, e o caminho é tentar de novo (`getSession`).
 */
export function SessionUnavailable() {
  const t = useTranslations("Auth.unavailable");
  const { retry, pending } = useRetrySession();
  return (
    <Notice
      kind="error"
      title={t("title")}
      action={
        <Button kind="tertiary" pending={pending} onClick={retry}>
          {t("retry")}
        </Button>
      }
    >
      {t("text")}
    </Notice>
  );
}
