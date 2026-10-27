"use client";

import { useTranslations } from "next-intl";
import { useActionState, useState } from "react";

import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { Alert } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { Field } from "@/shared/ui/field";
import { Input } from "@/shared/ui/input";
import { useEnterAccount } from "../hooks/use-enter-account";

/** Nome de usuário e senha; entrando, vai para `returnTo`. */
export function LoginForm({ returnTo }: { returnTo: string }) {
  const t = useTranslations("Auth");
  const errorMessage = useApiErrorMessage();
  const enter = useEnterAccount();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");

  const [error, submit, pending] = useActionState(async () => {
    const result = await callApi(
      api.POST("/auth/login", { body: { username, password } }),
    );
    if (!result.ok) return errorMessage(result.code, result.params);
    enter(returnTo);
    return undefined;
  }, undefined);

  return (
    <form action={submit} className="flex flex-col gap-4">
      <Field label={t("username")}>
        <Input
          required
          autoComplete="username"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
        />
      </Field>
      <Field label={t("password")}>
        <Input
          required
          type="password"
          autoComplete="current-password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
      </Field>
      {error && <Alert>{error}</Alert>}
      <Button type="submit" disabled={pending} className="self-start">
        {t("submit")}
      </Button>
    </form>
  );
}
