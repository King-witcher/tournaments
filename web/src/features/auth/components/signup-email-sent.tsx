"use client";

import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import { Notice } from "@/shared/ui/notice";
import { SignupCodeForm } from "./signup-code-form";

type Props = {
  /** Para onde o email foi, na forma canônica que a API devolveu. */
  email: string;
  /** O pedido do cadastro, para reenviar igual. */
  body: components["schemas"]["Signup"];
  /** Para onde ir depois de criar a conta pelo código. */
  returnTo: string;
  /** O reenvio deu certo (com o endereço canônico de novo). */
  onResent: (email: string) => void;
  /** Volta ao formulário, com os campos como estavam. */
  onBack: () => void;
};

/**
 * Depois do cadastro: para onde foram o código e o link, o campo do código
 * (que recebe o foco ao aparecer, no lugar do botão que sumiu) e quanto eles
 * valem. "Reenviar email" repete o mesmo pedido (código e link novos
 * derrubam os anteriores, e o campo volta vazio; cedo demais, diz quantos
 * segundos esperar); "Usar outro email" volta ao formulário.
 */
export function SignupEmailSent({
  email,
  body,
  returnTo,
  onResent,
  onBack,
}: Props) {
  const t = useTranslations("Signup.sent");
  const { run, pending, error } = useApiAction();
  const [sends, setSends] = useState(0);

  const resend = () =>
    run(
      () => api.POST("/auth/signup", { body }),
      (data) => {
        setSends((n) => n + 1);
        onResent(data.email);
      },
    );

  return (
    <Stack gap={6}>
      <Notice kind="success" title={t("title")}>
        {t("text", { email })}
      </Notice>
      <SignupCodeForm
        key={sends}
        email={email}
        returnTo={returnTo}
        focusOnMount={sends === 0}
      />
      <ActionWithNote
        error={error}
        note={sends > 0 && !pending && !error && t("resent")}
      >
        <Button kind="tertiary" pending={pending} onClick={resend}>
          {t("resend")}
        </Button>
      </ActionWithNote>
      <Button kind="ghost" onClick={onBack}>
        {t("back")}
      </Button>
    </Stack>
  );
}
