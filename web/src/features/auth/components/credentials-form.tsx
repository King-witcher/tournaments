"use client";

import { useTranslations } from "next-intl";
import { useActionState, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { Alert } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { Field } from "@/shared/ui/field";
import { Input } from "@/shared/ui/input";
import { useEnterAccount } from "../hooks/use-enter-account";

type Props =
  /** Cria a conta e entra, indo para `returnTo`. */
  | { mode: "signup"; returnTo: string }
  /** Dá nome de usuário e senha à conta logada. */
  | { mode: "add" };

/** Nome de usuário, senha e a senha repetida. */
export function CredentialsForm(props: Props) {
  const t = useTranslations("Auth");
  const tSignup = useTranslations("Signup");
  const tAccount = useTranslations("Account.password");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const enter = useEnterAccount();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");

  const [error, submit, pending] = useActionState(async () => {
    if (password !== confirm) return t("passwordsDiffer");
    const body = { username, password };
    if (props.mode === "signup") {
      const result = await callApi(api.POST("/auth/signup", { body }));
      if (!result.ok) return errorMessage(result.code, result.params);
      enter(props.returnTo);
    } else {
      const result = await callApi(api.POST("/auth/password", { body }));
      if (!result.ok) return errorMessage(result.code, result.params);
      router.refresh();
    }
    return undefined;
  }, undefined);

  return (
    <form action={submit} className="flex flex-col gap-4">
      <Field label={t("username")} hint={t("usernameHint")}>
        <Input
          required
          autoComplete="username"
          minLength={3}
          maxLength={20}
          pattern="[A-Za-z0-9_.\-]{3,20}"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
        />
      </Field>
      <Field label={t("password")} hint={t("passwordHint")}>
        <Input
          required
          type="password"
          autoComplete="new-password"
          minLength={8}
          maxLength={128}
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
      </Field>
      <Field label={t("confirmPassword")}>
        <Input
          required
          type="password"
          autoComplete="new-password"
          value={confirm}
          onChange={(e) => setConfirm(e.target.value)}
        />
      </Field>
      {error && <Alert>{error}</Alert>}
      <Button type="submit" pending={pending} className="self-start">
        {props.mode === "signup" ? tSignup("submit") : tAccount("create")}
      </Button>
    </form>
  );
}
