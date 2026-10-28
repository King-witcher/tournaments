"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useActionState, useId, useState } from "react";

import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { FormError } from "@/shared/ui/form-error";
import { Notice } from "@/shared/ui/notice";
import { TextInput } from "@/shared/ui/text-input";

/**
 * Pede o link de recuperação pelo nome de usuário ou pelo email. A API
 * responde igual exista a conta ou não, então a resposta é neutra: "se
 * houver uma conta com senha e email confirmado, o link foi". Enviado, o
 * aviso fica no lugar do formulário (e recebe o foco); "Pedir outro link"
 * volta ao campo, com o que foi digitado.
 */
export function ForgotPasswordForm() {
  const t = useTranslations("Login.forgot");
  const errorMessage = useApiErrorMessage();
  const inputId = useId();
  const sentId = useId();
  const [login, setLogin] = useState("");
  const [sentFor, setSentFor] = useState<string>();

  const [error, submit, pending] = useActionState(async () => {
    const outcome = await callApi(
      api.POST("/auth/password/reset", { body: { login } }),
    );
    if (!outcome.ok) return errorMessage(outcome.code, outcome.params);
    setSentFor(login.trim());
    focusIfLost(sentId);
    return undefined;
  }, undefined);

  if (sentFor !== undefined) {
    return (
      <div id={sentId} tabIndex={-1}>
        <Stack gap={5}>
          <Notice kind="success" title={t("sentTitle")}>
            {t("sent", { login: sentFor })}
          </Notice>
          <Button
            kind="tertiary"
            onClick={() => {
              setSentFor(undefined);
              focusIfLost(inputId);
            }}
          >
            {t("again")}
          </Button>
        </Stack>
      </div>
    );
  }

  return (
    <Form action={submit}>
      <Stack gap={6}>
        <TextInput
          id={inputId}
          labelText={t("login")}
          required
          autoComplete="username"
          autoCapitalize="none"
          spellCheck={false}
          maxLength={254}
          value={login}
          onChange={(e) => setLogin(e.target.value)}
        />
        {error && <FormError>{error}</FormError>}
        <Button type="submit" pending={pending}>
          {t("submit")}
        </Button>
      </Stack>
    </Form>
  );
}
