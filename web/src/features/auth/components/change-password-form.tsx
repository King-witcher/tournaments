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

type State = { saved: boolean; error?: string };

/** Troca o nome de usuário e/ou a senha, confirmando a senha atual. */
export function ChangePasswordForm({ username }: { username: string }) {
  const t = useTranslations("Auth");
  const tAccount = useTranslations("Account");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const [newUsername, setNewUsername] = useState(username);
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [current, setCurrent] = useState("");

  const [state, submit, pending] = useActionState<State>(
    async () => {
      if (password !== confirm) {
        return { saved: false, error: t("passwordsDiffer") };
      }
      const result = await callApi(
        api.PATCH("/auth/password", {
          body: {
            current_password: current,
            username: newUsername.trim() === username ? undefined : newUsername,
            password: password || undefined,
          },
        }),
      );
      if (!result.ok) {
        return {
          saved: false,
          error: errorMessage(result.code, result.params),
        };
      }
      setPassword("");
      setConfirm("");
      setCurrent("");
      router.refresh();
      return { saved: true };
    },
    { saved: false },
  );

  return (
    <form action={submit} className="flex flex-col gap-4">
      <Field label={t("username")} hint={t("usernameHint")}>
        <Input
          required
          autoComplete="username"
          minLength={3}
          maxLength={20}
          pattern="[A-Za-z0-9_.\-]{3,20}"
          value={newUsername}
          onChange={(e) => setNewUsername(e.target.value)}
        />
      </Field>
      <Field
        label={tAccount("password.newPassword")}
        hint={tAccount("password.newPasswordHint")}
      >
        <Input
          type="password"
          autoComplete="new-password"
          minLength={8}
          maxLength={128}
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
      </Field>
      {password && (
        <Field label={t("confirmPassword")}>
          <Input
            required
            type="password"
            autoComplete="new-password"
            value={confirm}
            onChange={(e) => setConfirm(e.target.value)}
          />
        </Field>
      )}
      <Field label={tAccount("currentPassword")}>
        <Input
          required
          type="password"
          autoComplete="current-password"
          value={current}
          onChange={(e) => setCurrent(e.target.value)}
        />
      </Field>
      {state.error && <Alert>{state.error}</Alert>}
      <div className="flex items-center gap-4">
        <Button type="submit" disabled={pending}>
          {tAccount("password.save")}
        </Button>
        {state.saved && !pending && (
          <span className="text-sm opacity-70">
            {tAccount("password.saved")}
          </span>
        )}
      </div>
    </form>
  );
}
