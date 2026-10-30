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
import {
  type FormResult,
  fieldId,
  formError,
  invalidProps,
} from "@/shared/lib/form-result";
import { usernameChars } from "@/shared/lib/input-filters";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { Notice } from "@/shared/ui/notice";
import { PasswordInput } from "@/shared/ui/password-input";
import { TextInput } from "@/shared/ui/text-input";
import { type AuthField, errorField } from "../lib/auth-field";

/**
 * Troca o nome de usuário e/ou a senha, confirmando a senha atual. A senha
 * repetida só aparece quando há senha nova. O erro de um campo (senha atual
 * errada, nome em uso) vai nele, que recebe o foco.
 */
export function ChangePasswordForm({ username }: { username: string }) {
  const t = useTranslations("Auth");
  const tAccount = useTranslations("Account");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const prefix = useId();
  const { ref, canSubmit } = useFormValidity();
  const [newUsername, setNewUsername] = useState(username);
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [current, setCurrent] = useState("");

  const [result, submit, pending] = useActionState<FormResult<AuthField>>(
    async () => {
      // Sem senha nova, a repetida nem aparece: não há o que comparar.
      if (password && password !== confirm) {
        return { error: t("passwordsDiffer"), field: "confirm" };
      }
      const outcome = await callApi(
        api.PATCH("/auth/password", {
          body: {
            current_password: current,
            username: newUsername.trim() === username ? undefined : newUsername,
            password: password || undefined,
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
      setConfirm("");
      setCurrent("");
      router.refresh();
      return { saved: true };
    },
    {},
  );
  useFocusInvalid(prefix, result);
  const general = formError(result);

  return (
    <Form ref={ref} action={submit}>
      <Stack gap={6}>
        <TextInput
          id={fieldId(prefix, "username")}
          labelText={t("username")}
          helperText={t("usernameAbout")}
          required
          autoComplete="username"
          minLength={3}
          maxLength={20}
          pattern="[A-Za-z0-9_.\-]{3,20}"
          patternMessage={t("usernameInvalid")}
          filter={usernameChars}
          value={newUsername}
          onChange={(e) => setNewUsername(e.target.value)}
          {...invalidProps(result, "username")}
        />
        <PasswordInput
          id={fieldId(prefix, "password")}
          labelText={tAccount("password.newPassword")}
          helperText={tAccount("password.newPasswordHint")}
          autoComplete="new-password"
          minLength={8}
          maxLength={128}
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          {...invalidProps(result, "password")}
        />
        {password && (
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
        )}
        <PasswordInput
          id={fieldId(prefix, "current")}
          labelText={tAccount("currentPassword")}
          required
          autoComplete="current-password"
          value={current}
          onChange={(e) => setCurrent(e.target.value)}
          {...invalidProps(result, "current")}
        />
        {general && <FormError>{general}</FormError>}
        {result.saved && !pending && (
          <Notice kind="success">{tAccount("password.saved")}</Notice>
        )}
        <FormActions pending={pending} disabled={!canSubmit} />
      </Stack>
    </Form>
  );
}
