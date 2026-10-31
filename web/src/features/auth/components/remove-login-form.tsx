"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useRef, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { ConfirmModal } from "@/shared/ui/confirm-modal";
import { ErrorText } from "@/shared/ui/error-text";
import { PasswordInput } from "@/shared/ui/password-input";
import { LOGIN_SECTION } from "../lib/login-sections";
import type { Provider } from "../lib/providers";

type Props = {
  /** O provedor a desvincular (a senha sai pelo `RemovePasswordButton`). */
  login: Provider;
  /**
   * A conta tem senha, e a API pede a atual para desvincular. Sem senha, o
   * formulário é só o botão.
   */
  needsPassword?: boolean;
};

/** Erros que são da senha digitada (vão no campo); o resto vai embaixo do botão. */
const PASSWORD_ERRORS = new Set([
  "auth.wrong_password",
  "auth.too_many_attempts",
]);

/**
 * Desvincula um provedor da conta. Com senha na conta, pede a atual (a API confere)
 * e, ao enviar, o `ConfirmModal` diz o que a conta perde; só o "sim" chama a
 * API. O foco volta para onde estava ao fechar e, quando o login sai, vai
 * para o bloco dele (o formulário some no refresh).
 */
export function RemoveLoginForm({ login, needsPassword = true }: Props) {
  const t = useTranslations("Account");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const { ref, canSubmit } = useFormValidity();
  const [current, setCurrent] = useState("");
  const [confirming, setConfirming] = useState(false);
  const [passwordError, setPasswordError] = useState(false);
  const opener = useRef<HTMLElement | null>(null);
  const text = {
    action: t(`${login}.unlink`),
    hint: t(`${login}.unlinkHint`),
    question: t(`${login}.unlinkConfirm`),
  };

  const close = () => {
    setConfirming(false);
    // O Modal sai da árvore e o foco cairia no body.
    requestAnimationFrame(() => opener.current?.focus());
  };

  const remove = () => {
    const body = needsPassword ? { current_password: current } : {};
    const done = () => {
      setCurrent("");
      setPasswordError(false);
      router.refresh();
      focusIfLost(LOGIN_SECTION[login]);
    };
    // Sem o campo, todo erro vai embaixo do botão.
    const failed = (code: string) =>
      setPasswordError(needsPassword && PASSWORD_ERRORS.has(code));
    if (login === "discord") {
      run(() => api.DELETE("/auth/discord", { body }), done, failed);
    } else {
      run(() => api.DELETE("/auth/google", { body }), done, failed);
    }
  };

  return (
    <Form
      ref={ref}
      onSubmit={(event) => {
        event.preventDefault();
        if (pending) return;
        // O botão ou, com Enter, o campo: o foco volta para lá ao fechar.
        opener.current = document.activeElement as HTMLElement | null;
        setConfirming(true);
      }}
    >
      <Stack gap={5}>
        {needsPassword && (
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
        )}
        <Button
          type="submit"
          kind="danger--tertiary"
          pending={pending}
          disabled={!canSubmit}
        >
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
