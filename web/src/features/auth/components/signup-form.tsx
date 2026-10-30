"use client";

import { Form, Stack } from "@carbon/react";
import { useLocale, useTranslations } from "next-intl";
import { useActionState, useId, useState } from "react";

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
import { rememberSignupReturnTo } from "../lib/signup-return-to";
import { SignupEmailSent } from "./signup-email-sent";

/**
 * Cadastro com nome de usuário, email e senha (repetida). A conta só nasce
 * com o código ou o link mandados ao email: no 202, o formulário dá lugar ao
 * "confira seu email" (`SignupEmailSent`, com o campo do código), com o
 * endereço na forma canônica que a API devolveu. Os campos continuam no
 * estado, para reenviar o mesmo pedido ou voltar e trocar o email. O erro de
 * um campo vai nele, que recebe o foco; `returnTo` fica guardado para depois
 * do link, que pode abrir em outra aba.
 */
export function SignupForm({ returnTo }: { returnTo: string }) {
  const t = useTranslations("Auth");
  const tSignup = useTranslations("Signup");
  const locale = useLocale();
  const errorMessage = useApiErrorMessage();
  const prefix = useId();
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [sentTo, setSentTo] = useState<string>();
  const body = { username, email, password, locale };

  const [result, submit, pending] = useActionState<FormResult<AuthField>>(
    async () => {
      if (password !== confirm) {
        return { error: t("passwordsDiffer"), field: "confirm" };
      }
      const outcome = await callApi(api.POST("/auth/signup", { body }));
      if (!outcome.ok) {
        return {
          error: errorMessage(outcome.code, outcome.params),
          field: errorField(outcome.code),
        };
      }
      rememberSignupReturnTo(returnTo);
      setSentTo(outcome.data.email);
      return {};
    },
    {},
  );
  useFocusInvalid(prefix, result);
  const general = formError(result);

  if (sentTo) {
    return (
      <SignupEmailSent
        email={sentTo}
        body={body}
        returnTo={returnTo}
        onResent={setSentTo}
        onBack={() => {
          setSentTo(undefined);
          // O formulário volta no lugar do botão: o foco vai para o email.
          focusIfLost(fieldId(prefix, "email"));
        }}
      />
    );
  }

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
        <TextInput
          id={fieldId(prefix, "email")}
          type="email"
          labelText={t("email")}
          helperText={t("emailHint")}
          required
          autoComplete="email"
          maxLength={254}
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          {...invalidProps(result, "email")}
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
          {tSignup("submit")}
        </Button>
      </Stack>
    </Form>
  );
}
