"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useActionState, useId, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFocusInvalid } from "@/shared/hooks/use-focus-invalid";
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
import { TextInput } from "@/shared/ui/text-input";
import { type AuthField, errorField } from "../lib/auth-field";
import { LOGIN_SECTION } from "../lib/login-sections";

/**
 * Dá nome de usuário e senha à conta logada (que entra pelo Discord, que
 * continua sendo a recuperação: aqui não se pede email). Nome, senha e a
 * senha repetida. O erro de um campo (nome em uso, senhas diferentes) vai
 * nele, que recebe o foco; o resto, acima do botão. O cadastro com senha é o
 * `SignupForm`.
 */
export function CredentialsForm() {
  const t = useTranslations("Auth");
  const tAccount = useTranslations("Account.password");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const prefix = useId();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");

  const [result, submit, pending] = useActionState<FormResult<AuthField>>(
    async () => {
      if (password !== confirm) {
        return { error: t("passwordsDiffer"), field: "confirm" };
      }
      const outcome = await callApi(
        api.POST("/auth/password", { body: { username, password } }),
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
    <Form action={submit}>
      <Stack gap={6}>
        <TextInput
          id={fieldId(prefix, "username")}
          labelText={t("username")}
          helperText={t("usernameHint")}
          required
          autoComplete="username"
          minLength={3}
          maxLength={20}
          pattern="[A-Za-z0-9_.\-]{3,20}"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          {...invalidProps(result, "username")}
        />
        <PasswordInput
          id={fieldId(prefix, "password")}
          labelText={t("password")}
          helperText={t("passwordHint")}
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
          autoComplete="new-password"
          value={confirm}
          onChange={(e) => setConfirm(e.target.value)}
          {...invalidProps(result, "confirm")}
        />
        {general && <FormError>{general}</FormError>}
        <Button type="submit" pending={pending}>
          {tAccount("create")}
        </Button>
      </Stack>
    </Form>
  );
}
