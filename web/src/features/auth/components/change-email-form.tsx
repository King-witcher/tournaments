"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useActionState, useId, useState } from "react";

import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFocusInvalid } from "@/shared/hooks/use-focus-invalid";
import {
  type FormResult,
  fieldId,
  formError,
  invalidProps,
} from "@/shared/lib/form-result";
import { Button } from "@/shared/ui/button";
import { FormError } from "@/shared/ui/form-error";
import { Notice } from "@/shared/ui/notice";
import { PasswordInput } from "@/shared/ui/password-input";
import { TextInput } from "@/shared/ui/text-input";
import { type AuthField, errorField } from "../lib/auth-field";

type Props = {
  /** O email confirmado de agora, se houver (o rótulo vira "Email novo"). */
  current?: string;
  /** A conta tem senha: a API pede a atual para trocar o email. */
  hasPassword: boolean;
};

/**
 * Para onde foi o link, na forma canônica que a API devolveu, e o que foi
 * digitado, para o aviso sumir quando o campo é editado de novo.
 */
type Result = FormResult<AuthField> & { sentTo?: string; typed?: string };

/**
 * Cadastra ou troca o email da conta. A API manda um link ao endereço novo, e
 * o email só muda quando ele é aberto (`/email/confirm`); até lá fica o de
 * antes, por isso não há refresh. Com senha na conta, pede a atual. O erro de
 * um campo (email inválido ou de outra conta, senha errada) vai nele, que
 * recebe o foco.
 */
export function ChangeEmailForm({ current, hasPassword }: Props) {
  const t = useTranslations("Account.email");
  const tAccount = useTranslations("Account");
  const errorMessage = useApiErrorMessage();
  const prefix = useId();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");

  const [result, submit, pending] = useActionState<Result>(async () => {
    const outcome = await callApi(
      api.PUT("/auth/email", {
        body: {
          email,
          current_password: hasPassword ? password : undefined,
        },
      }),
    );
    if (!outcome.ok) {
      return {
        error: errorMessage(outcome.code, outcome.params),
        field: errorField(outcome.code),
      };
    }
    setPassword("");
    return { sentTo: outcome.data.email, typed: email };
  }, {});
  useFocusInvalid(prefix, result);
  const general = formError(result);
  const sent = !pending && result.typed === email ? result.sentTo : undefined;

  return (
    <Form action={submit}>
      <Stack gap={6}>
        <TextInput
          id={fieldId(prefix, "email")}
          type="email"
          labelText={current ? t("newLabel") : t("label")}
          helperText={t("hint")}
          required
          autoComplete="email"
          maxLength={254}
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          {...invalidProps(result, "email")}
        />
        {hasPassword && (
          <PasswordInput
            id={fieldId(prefix, "current")}
            labelText={tAccount("currentPassword")}
            required
            autoComplete="current-password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            {...invalidProps(result, "current")}
          />
        )}
        {general && <FormError>{general}</FormError>}
        {sent && (
          <Notice kind="success" title={t("sentTitle")}>
            {t("sent", { email: sent })}
          </Notice>
        )}
        <Button type="submit" pending={pending}>
          {current ? t("change") : t("add")}
        </Button>
      </Stack>
    </Form>
  );
}
