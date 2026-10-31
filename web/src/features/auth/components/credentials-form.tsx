"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useActionState, useId, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFocusInvalid } from "@/shared/hooks/use-focus-invalid";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import { focusIfLost } from "@/shared/lib/focus";
import {
  type FormResult,
  fieldId,
  formError,
  invalidProps,
} from "@/shared/lib/form-result";
import { Button } from "@/shared/ui/button";
import { FormError } from "@/shared/ui/form-error";
import { PasswordInput } from "@/shared/ui/password-input";
import { type AuthField, errorField } from "../lib/auth-field";
import { LOGIN_SECTION } from "../lib/login-sections";

/**
 * Dá senha à conta logada (que entra pelo Discord ou pelo Google e já tem o
 * email, que passa a ser o login: sem ele, o formulário nem aparece). Senha
 * e a senha repetida. O erro de um campo (senhas diferentes) vai nele, que
 * recebe o foco; o resto, acima do botão. O cadastro com senha é o
 * `SignupForm`.
 */
export function CredentialsForm() {
  const t = useTranslations("Auth");
  const tAccount = useTranslations("Account.password");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const prefix = useId();
  const { ref, canSubmit } = useFormValidity();
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");

  const [result, submit, pending] = useActionState<FormResult<AuthField>>(
    async () => {
      if (password !== confirm) {
        return { error: t("passwordsDiffer"), field: "confirm" };
      }
      const outcome = await callApi(
        api.POST("/auth/password", { body: { password } }),
      );
      if (!outcome.ok) {
        return {
          error: errorMessage(outcome.code, outcome.params),
          field: errorField(outcome.code),
        };
      }
      router.refresh();
      // O formulário some quando a conta passa a ter senha.
      focusIfLost(LOGIN_SECTION.password);
      return {};
    },
    {},
  );
  useFocusInvalid(prefix, result);
  const general = formError(result);

  return (
    <Form ref={ref} action={submit}>
      <Stack gap={6}>
        <PasswordInput
          id={fieldId(prefix, "password")}
          labelText={t("password")}
          required
          autoComplete="new-password"
          minLength={8}
          maxLength={128}
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          {...invalidProps(result, "password")}
        />
        <PasswordInput
          id={fieldId(prefix, "confirm")}
          labelText={t("confirmPassword")}
          required
          sameAs={password}
          autoComplete="new-password"
          value={confirm}
          onChange={(e) => setConfirm(e.target.value)}
          {...invalidProps(result, "confirm")}
        />
        {general && <FormError>{general}</FormError>}
        <Button type="submit" pending={pending} disabled={!canSubmit}>
          {tAccount("create")}
        </Button>
      </Stack>
    </Form>
  );
}
