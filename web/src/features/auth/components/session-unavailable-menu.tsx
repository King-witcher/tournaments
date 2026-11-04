"use client";

import { useTranslations } from "next-intl";

import { UserPanel } from "@/shared/ui/user-panel";
import { useRetrySession } from "../hooks/use-retry-session";

/**
 * O lugar da pessoa no cabeçalho quando a API não respondeu: em vez de
 * "Entrar", o que houve e "Tentar de novo". Tema e idioma continuam no painel.
 */
export function SessionUnavailableMenu() {
  const t = useTranslations("Auth.unavailable");
  const { retry, pending } = useRetrySession();
  return (
    <UserPanel
      label={t("title")}
      name={t("short")}
      items={[{ label: t("retry"), pending, onSelect: retry }]}
    />
  );
}
