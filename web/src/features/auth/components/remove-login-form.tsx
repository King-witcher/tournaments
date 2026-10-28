"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useRef, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { ConfirmModal } from "@/shared/ui/confirm-modal";
import { ErrorText } from "@/shared/ui/error-text";
import { PasswordInput } from "@/shared/ui/password-input";
import { LOGIN_SECTION } from "../lib/login-sections";

type Props = {
  /** O nome de usuário e a senha (`password`) ou o Discord (`discord`). */
  login: "password" | "discord";
};

/** Erros que são da senha digitada (vão no campo); o resto vai embaixo do botão. */
const PASSWORD_ERRORS = new Set([
  "auth.wrong_password",
  "auth.too_many_attempts",
]);

/**
 * Remove um login da conta. Pede a senha atual (a API confere) e, ao enviar,
 * o `ConfirmModal` diz o que a conta perde; só o "sim" chama a API. O foco
 * volta para onde estava ao fechar e, quando o login sai, vai para o bloco
 * dele (o formulário some no refresh).
 */
export function RemoveLoginForm({ login }: Props) {
  const t = useTranslations("Account");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const [current, setCurrent] = useState("");
  const [confirming, setConfirming] = useState(false);
  const [passwordError, setPasswordError] = useState(false);
  const opener = useRef<HTMLElement | null>(null);
  const text =
    login === "password"
      ? {
          action: t("password.remove"),
          hint: t("password.removeHint"),
          question: t("password.removeConfirm"),
        }
      : {
          action: t("discord.unlink"),
          hint: t("discord.unlinkHint"),
          question: t("discord.unlinkConfirm"),
        };

  const close = () => {
    setConfirming(false);
    // O Modal sai da árvore e o foco cairia no body.
    requestAnimationFrame(() => opener.current?.focus());
  };

  const remove = () => {
    const body = { current_password: current };
    const done = () => {
      setCurrent("");
      setPasswordError(false);
      router.refresh();
      focusIfLost(LOGIN_SECTION[login]);
    };
    const failed = (code: string) =>
      setPasswordError(PASSWORD_ERRORS.has(code));
    if (login === "password") {
      run(() => api.DELETE("/auth/password", { body }), done, failed);
    } else {
      run(() => api.DELETE("/auth/discord", { body }), done, failed);
    }
  };

  return (
    <Form
      onSubmit={(event) => {
        event.preventDefault();
        if (pending) return;
        // O botão ou, com Enter, o campo: o foco volta para lá ao fechar.
        opener.current = document.activeElement as HTMLElement | null;
        setConfirming(true);
      }}
    >
      <Stack gap={5}>
        <PasswordInput
          labelText={t("currentPassword")}
          helperText={text.hint}
          required
          autoComplete="current-password"
          value={current}
          onChange={(e) => setCurrent(e.target.value)}
          invalid={passwordError && error !== undefined}
          invalidText={error}
        />
        <Button type="submit" kind="danger--tertiary" pending={pending}>
          {text.action}
        </Button>
        {error && !passwordError && <ErrorText>{error}</ErrorText>}
      </Stack>
      <ConfirmModal
        open={confirming}
        onClose={close}
        onConfirm={() => {
          close();
          remove();
        }}
        title={text.action}
        confirmLabel={text.action}
        destructive
      >
        {text.question}
      </ConfirmModal>
    </Form>
  );
}
