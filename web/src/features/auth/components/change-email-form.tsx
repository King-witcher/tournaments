"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useActionState, useId, useState } from "react";

import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFocusInvalid } from "@/shared/hooks/use-focus-invalid";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import {
  type FormResult,
  fieldId,
  formError,
  invalidProps,
} from "@/shared/lib/form-result";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { Notice } from "@/shared/ui/notice";
import { PasswordInput } from "@/shared/ui/password-input";
import { TextInput } from "@/shared/ui/text-input";
import { type AuthField, errorField } from "../lib/auth-field";
import { RemoveEmailButton } from "./remove-email-button";

type Props = {
  /** O email confirmado de agora, se houver: vem no campo, para editar. */
  current?: string;
  /** A conta tem senha: a API pede a atual para trocar o email, e ele não sai. */
  hasPassword: boolean;
};

/**
 * Para onde foi o link, na forma canônica que a API devolveu, e o email que
 * foi enviado, para o aviso e o erro sumirem quando o campo muda (inclusive
 * pelo "Cancelar").
 */
type Result = FormResult<AuthField> & { sentTo?: string; typed?: string };

/**
 * Cadastra, troca ou remove o email da conta. O campo vem com o de agora; a
 * API manda um link ao endereço novo, e o email só muda quando ele é aberto
 * (`/email/confirm`), por isso não há refresh. Com senha na conta, a atual
 * aparece quando o campo muda. "Cancelar" volta o campo ao de agora. O erro
 * de um campo (email inválido, igual ao de agora ou de outra conta, senha
 * errada) vai nele, que recebe o foco.
 */
export function ChangeEmailForm({ current, hasPassword }: Props) {
  const t = useTranslations("Account.email");
  const tAccount = useTranslations("Account");
  const errorMessage = useApiErrorMessage();
  const prefix = useId();
  const { ref, canSubmit } = useFormValidity();
  const [email, setEmail] = useState(current ?? "");
  const [password, setPassword] = useState("");
  // O email gravado é minúsculo: mudar só a caixa não é trocar.
  const changed = email.trim().toLowerCase() !== (current ?? "");
  const askPassword = hasPassword && changed;

  const [result, submit, pending] = useActionState<Result>(async () => {
    if (!changed) {
      return {
        error: errorMessage("auth.email_unchanged"),
        field: "email",
        typed: email,
      };
    }
    const outcome = await callApi(
      api.PUT("/auth/email", {
        body: {
          email,
          current_password: askPassword ? password : undefined,
        },
      }),
    );
    if (!outcome.ok) {
      return {
        error: errorMessage(outcome.code, outcome.params),
        field: errorField(outcome.code),
        typed: email,
      };
    }
    setPassword("");
    return { sentTo: outcome.data.email, typed: email };
  }, {});
  useFocusInvalid(prefix, result);
  // O resultado vale para o email enviado; com o campo mudado, sai.
  const shown: Result = !pending && result.typed === email ? result : {};
  const general = formError(shown);
  const sent = shown.sentTo;
  const editing = Boolean(current) && changed && !sent;

  return (
    <Form ref={ref} action={submit}>
      <Stack gap={6}>
        <TextInput
          id={fieldId(prefix, "email")}
          type="email"
          labelText={t("label")}
          required
          autoComplete="email"
          maxLength={254}
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          {...invalidProps(shown, "email")}
        />
        {askPassword && (
          <PasswordInput
            id={fieldId(prefix, "current")}
            labelText={tAccount("currentPassword")}
            required
            autoComplete="current-password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            {...invalidProps(shown, "current")}
          />
        )}
        {general && <FormError>{general}</FormError>}
        {sent && (
          <Notice kind="success" title={t("sentTitle")}>
            {current
              ? t("sentChange", { email: sent, current })
              : t("sent", { email: sent })}
          </Notice>
        )}
        <FormActions
          submitLabel={current ? t("change") : t("add")}
          pending={pending}
          disabled={!canSubmit}
          onCancel={
            editing
              ? () => {
                  setEmail(current ?? "");
                  setPassword("");
                }
              : undefined
          }
          extra={current && <RemoveEmailButton blocked={hasPassword} />}
        />
      </Stack>
    </Form>
  );
}
