"use client";

import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import { Notice } from "@/shared/ui/notice";
import { useCooldown } from "../hooks/use-cooldown";
import { EmailLoginCodeForm } from "./email-login-code-form";

/**
 * A espera da API entre dois envios ao mesmo endereço. Fora dela o email só não
 * sai, sem erro (a resposta não pode revelar quem tem conta), então o botão
 * espera aqui, para não prometer um envio que não acontece.
 */
const RESEND_AFTER_SECS = 60;

type Props = {
  /** O email do pedido, como foi digitado. */
  email: string;
  returnTo: string;
  /** Volta ao formulário, com o email como estava. */
  onBack: () => void;
};

/**
 * Depois do pedido: a resposta neutra, o campo do código (que recebe o foco, no
 * lugar do botão que sumiu), reenviar (código e link novos derrubam os
 * anteriores, e o campo volta vazio) e usar outro email.
 */
export function EmailLoginSent({ email, returnTo, onBack }: Props) {
  const t = useTranslations("Login.email");
  const { run, pending, error } = useApiAction();
  const [sends, setSends] = useState(0);
  const cooldown = useCooldown(RESEND_AFTER_SECS);

  const resend = () =>
    run(
      () => api.POST("/auth/email/login", { body: { email } }),
      () => {
        setSends((n) => n + 1);
        cooldown.restart();
      },
    );

  return (
    <Stack gap={6}>
      <Notice kind="success" title={t("sentTitle")}>
        {t("sent", { email })}
      </Notice>
      <EmailLoginCodeForm
        key={sends}
        email={email}
        returnTo={returnTo}
        focusOnMount={sends === 0}
      />
      <ActionWithNote error={error}>
        <Button
          kind="tertiary"
          pending={pending}
          disabled={cooldown.left > 0}
          onClick={resend}
        >
          {cooldown.left > 0
            ? t("resendIn", { seconds: cooldown.left })
            : t("resend")}
        </Button>
      </ActionWithNote>
      <Button kind="ghost" onClick={onBack}>
        {t("back")}
      </Button>
    </Stack>
  );
}
