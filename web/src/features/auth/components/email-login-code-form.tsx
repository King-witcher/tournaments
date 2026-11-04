"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useActionState, useId, useState } from "react";

import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFocusInvalid } from "@/shared/hooks/use-focus-invalid";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import type { FormResult } from "@/shared/lib/form-result";
import { Button } from "@/shared/ui/button";
import { useCodeAutoSubmit } from "../hooks/use-code-auto-submit";
import { useEnterAccount } from "../hooks/use-enter-account";
import { useNewPassword } from "../hooks/use-new-password";
import { afterEmailLogin } from "../lib/after-email-login";
import { type AuthField, errorField } from "../lib/auth-field";
import { CODE_LENGTH, CodeInput } from "./code-input";
import { NewPasswordFields } from "./new-password-fields";

type Props = {
  /** O email do pedido, como foi digitado: a API o põe na forma canônica. */
  email: string;
  returnTo: string;
  /** Leva o foco ao campo ao aparecer (a tela nova, não o reenvio). */
  focusOnMount: boolean;
};

/** O erro do último envio e o código que o causou. */
type Result = FormResult<AuthField> & { code?: string };

/**
 * O código de 6 dígitos para entrar por email, com a senha nova opcional. Envia
 * sozinho ao completar, a não ser com a senha nova aberta, que espera o botão.
 * O erro da senha vai no campo dela; o resto, no do código, até ele mudar.
 */
export function EmailLoginCodeForm({ email, returnTo, focusOnMount }: Props) {
  const t = useTranslations("Auth");
  const errorMessage = useApiErrorMessage();
  const enter = useEnterAccount();
  const prefix = useId();
  const { ref: form, canSubmit } = useFormValidity();
  const newPassword = useNewPassword();
  const [code, setCode] = useState("");
  // Enquanto a página da conta carrega, o botão continua pendente.
  const [leaving, setLeaving] = useState(false);

  const [result, submit, pending] = useActionState<Result, FormData>(
    async (_, data) => {
      const typed = String(data.get("code") ?? "");
      if (typed.length !== CODE_LENGTH) {
        return { error: t("code.incomplete"), code: typed };
      }
      if (newPassword.differs) {
        return { error: t("passwordsDiffer"), field: "confirm", code: typed };
      }
      const outcome = await callApi(
        api.POST("/auth/email/login/code", {
          body: { email, code: typed, password: newPassword.value },
        }),
      );
      if (!outcome.ok) {
        return {
          error: errorMessage(outcome.code, outcome.params),
          field: errorField(outcome.code),
          code: typed,
        };
      }
      setLeaving(true);
      enter(
        afterEmailLogin(
          outcome.data.deletion_cancelled,
          newPassword.value !== undefined,
          returnTo,
        ),
      );
      return {};
    },
    {},
  );
  useFocusInvalid(prefix, result);
  useCodeAutoSubmit(code, form, !newPassword.open);
  const passwordError =
    result.field === "password" || result.field === "confirm";

  return (
    <Form ref={form} action={submit}>
      <Stack gap={5}>
        <CodeInput
          value={code}
          onChange={setCode}
          error={
            !passwordError && result.code === code ? result.error : undefined
          }
          focusOnMount={focusOnMount}
        />
        <NewPasswordFields
          state={newPassword}
          prefix={prefix}
          result={result}
        />
        <Button
          type="submit"
          disabled={!canSubmit}
          pending={pending || leaving}
        >
          {t("submit")}
        </Button>
      </Stack>
    </Form>
  );
}
