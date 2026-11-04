"use client";

import { useTranslations } from "next-intl";

import { useRetrySession } from "@/features/auth/hooks/use-retry-session";
import { Button } from "@/shared/ui/button";

/**
 * "Tentar de novo" no convite que não carregou porque a API não respondeu (o
 * pedido acorda o servidor, e o seguinte costuma dar certo).
 */
export function RetryInviteButton() {
  const t = useTranslations("Errors");
  const { retry, pending } = useRetrySession();
  return (
    <Button kind="tertiary" pending={pending} onClick={retry}>
      {t("retry")}
    </Button>
  );
}
