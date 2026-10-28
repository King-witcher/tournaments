"use client";

import { InlineLoading, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";

import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import { Notice } from "@/shared/ui/notice";
import { usePush } from "../hooks/use-push";

/**
 * Os avisos com o site fechado neste navegador, na aba Avisos da conta: o
 * estado (ativado, desligado, bloqueado, sem suporte, iPhone sem o site
 * instalado, desligado no servidor) e o botão de ativar ou desativar. A
 * permissão só é pedida no clique.
 */
export function PushSettings() {
  const t = useTranslations("Notifications.push");
  const { status, pending, error, enable, disable } = usePush();

  switch (status) {
    case "loading":
      return <InlineLoading description={t("loading")} />;
    case "unsupported":
      return <Notice>{t("unsupported")}</Notice>;
    case "install":
      return <Notice title={t("installTitle")}>{t("install")}</Notice>;
    case "server_off":
      return <Notice>{t("serverOff")}</Notice>;
    case "denied":
      return (
        <Notice kind="warning" title={t("deniedTitle")}>
          {t("denied")}
        </Notice>
      );
  }

  const on = status === "on";
  return (
    <Stack gap={5}>
      <p>{on ? t("on") : t("off")}</p>
      <ActionWithNote error={error}>
        {/* O mesmo botão nos dois estados: o foco fica nele ao trocar. */}
        <Button
          kind={on ? "tertiary" : "primary"}
          pending={pending}
          onClick={on ? disable : enable}
        >
          {on ? t("disable") : t("enable")}
        </Button>
      </ActionWithNote>
    </Stack>
  );
}
