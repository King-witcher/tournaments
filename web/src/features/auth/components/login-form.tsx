"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useActionState, useState } from "react";

import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { Button } from "@/shared/ui/button";
import { FormError } from "@/shared/ui/form-error";
import { Link } from "@/shared/ui/link";
import { PasswordInput } from "@/shared/ui/password-input";
import { TextInput } from "@/shared/ui/text-input";
import { useEnterAccount } from "../hooks/use-enter-account";
import { DELETION_CANCELLED } from "../lib/deletion-cancelled";

/**
 * Email e senha; entrando, vai para `returnTo`. O erro (senha errada,
 * tentativas demais) é do formulário todo: dizer qual campo errou revelaria
 * quem tem conta. Entrar numa conta com a exclusão agendada a cancela e leva
 * à conta, que avisa. Embaixo do botão, o link para recuperar a senha
 * (depois dele na ordem do Tab, para não ficar entre a senha e o "Entrar"),
 * que leva o email digitado.
 */
export function LoginForm({ returnTo }: { returnTo: string }) {
  const t = useTranslations("Auth");
  const errorMessage = useApiErrorMessage();
  const enter = useEnterAccount();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");

  const [error, submit, pending] = useActionState(async () => {
    const result = await callApi(
      api.POST("/auth/login", { body: { email, password } }),
    );
    if (!result.ok) return errorMessage(result.code, result.params);
    enter(result.data.deletion_cancelled ? DELETION_CANCELLED : returnTo);
    return undefined;
  }, undefined);

  return (
    <Form action={submit}>
      <Stack gap={6}>
        <TextInput
          type="email"
          labelText={t("email")}
          required
          autoComplete="username"
          maxLength={254}
          value={email}
          onChange={(e) => setEmail(e.target.value)}
        />
        <PasswordInput
          labelText={t("password")}
          required
          autoComplete="current-password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
        {error && <FormError>{error}</FormError>}
        <Button type="submit" pending={pending}>
          {t("submit")}
        </Button>
        <Link
          href={{
            pathname: "/login/forgot",
            query: email.trim() ? { email: email.trim() } : undefined,
          }}
        >
          {t("forgotPassword")}
        </Link>
      </Stack>
    </Form>
  );
}
