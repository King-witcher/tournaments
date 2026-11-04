"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { type ReactNode, useActionState, useId, useState } from "react";

import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { FormError } from "@/shared/ui/form-error";
import { TextInput } from "@/shared/ui/text-input";
import { EmailLoginSent } from "./email-login-sent";

type Props = {
  /** O que estava no login (`?email=`). */
  initialEmail: string;
  returnTo: string;
  /** As outras saídas (os provedores), que somem depois do pedido. */
  alternatives: ReactNode;
};

/**
 * Pede o email com o link e o código. A API responde igual exista a conta ou
 * não; respondido, a tela do código fica no lugar do formulário.
 */
export function EmailLogin({ initialEmail, returnTo, alternatives }: Props) {
  const t = useTranslations("Login.email");
  const errorMessage = useApiErrorMessage();
  const inputId = useId();
  const { ref, canSubmit } = useFormValidity();
  const [email, setEmail] = useState(initialEmail);
  const [sentTo, setSentTo] = useState<string>();

  const [error, submit, pending] = useActionState(async () => {
    const outcome = await callApi(
      api.POST("/auth/email/login", { body: { email } }),
    );
    if (!outcome.ok) return errorMessage(outcome.code, outcome.params);
    setSentTo(email.trim());
    return undefined;
  }, undefined);

  if (sentTo !== undefined) {
    return (
      <EmailLoginSent
        email={sentTo}
        returnTo={returnTo}
        onBack={() => {
          setSentTo(undefined);
          focusIfLost(inputId);
        }}
      />
    );
  }

  return (
    <Stack gap={6}>
      <Form ref={ref} action={submit}>
        <Stack gap={6}>
          <TextInput
            id={inputId}
            type="email"
            labelText={t("address")}
            required
            autoComplete="username"
            autoCapitalize="none"
            spellCheck={false}
            maxLength={254}
            value={email}
            onChange={(e) => setEmail(e.target.value)}
          />
          {error && <FormError>{error}</FormError>}
          <Button type="submit" pending={pending} disabled={!canSubmit}>
            {t("submit")}
          </Button>
        </Stack>
      </Form>
      {alternatives}
    </Stack>
  );
}
