"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import { Stack } from "@/shared/ui/carbon/stack";
import { Notice } from "@/shared/ui/notice";
import { TextInput } from "@/shared/ui/text-input";
import { EmailProofSent } from "./email-proof-sent";

/**
 * Na conta que só entra pelo email, no lugar do formulário do email: mudar os
 * logins pede antes o código mandado a ele, para quem pegou só a sessão não
 * trocar o único login. Pede o código e, mandado, mostra onde digitá-lo; certo,
 * a página recarrega com os formulários.
 */
export function EmailProofForm({ email }: { email: string }) {
  const t = useTranslations("Account.email");
  const { run, pending, error } = useApiAction();
  const [sends, setSends] = useState(0);
  const request = () =>
    run(
      () => api.POST("/auth/email/proof"),
      () => setSends((n) => n + 1),
    );

  if (sends > 0) {
    return (
      <EmailProofSent
        key={sends}
        email={email}
        pending={pending}
        error={error}
        onResend={request}
        focusOnMount={sends === 1}
      />
    );
  }
  return (
    <Stack gap={6}>
      <TextInput labelText={t("label")} value={email} readOnly />
      <Notice kind="info">{t("proofNeeded")}</Notice>
      <ActionWithNote error={error}>
        <Button pending={pending} onClick={request}>
          {t("proofSend")}
        </Button>
      </ActionWithNote>
    </Stack>
  );
}
