"use client";

import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useEffect, useRef, useState } from "react";

import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import { Notice } from "@/shared/ui/notice";

type Props = {
  /** Para onde o link foi, na forma canônica que a API devolveu. */
  email: string;
  /** O pedido do cadastro, para reenviar igual. */
  body: components["schemas"]["Signup"];
  /** O reenvio deu certo (com o endereço canônico de novo). */
  onResent: (email: string) => void;
  /** Volta ao formulário, com os campos como estavam. */
  onBack: () => void;
};

/**
 * Depois do cadastro: para onde foi o link, quanto ele vale e que a conta só
 * nasce quando ele é aberto. "Reenviar" repete o mesmo pedido (o link novo
 * derruba o anterior; cedo demais, diz quantos segundos esperar); "Usar
 * outro email" volta ao formulário. Recebe o foco ao aparecer, no lugar do
 * botão que sumiu.
 */
export function SignupEmailSent({ email, body, onResent, onBack }: Props) {
  const t = useTranslations("Signup.sent");
  const { run, pending, error } = useApiAction();
  const [resent, setResent] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    ref.current?.focus();
  }, []);

  const resend = () =>
    run(
      () => api.POST("/auth/signup", { body }),
      (data) => {
        setResent(true);
        onResent(data.email);
      },
      () => setResent(false),
    );

  return (
    <div ref={ref} tabIndex={-1}>
      <Stack gap={6}>
        <Notice kind="success" title={t("title")}>
          {t("text", { email })}
        </Notice>
        <ActionWithNote error={error} note={resent && !pending && t("resent")}>
          <Button kind="tertiary" pending={pending} onClick={resend}>
            {t("resend")}
          </Button>
        </ActionWithNote>
        <Button kind="ghost" onClick={onBack}>
          {t("back")}
        </Button>
      </Stack>
    </div>
  );
}
