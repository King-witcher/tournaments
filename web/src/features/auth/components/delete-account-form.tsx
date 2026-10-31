"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useRef, useState } from "react";

import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import { Button } from "@/shared/ui/button";
import { ConfirmModal } from "@/shared/ui/confirm-modal";
import { ErrorText } from "@/shared/ui/error-text";
import { PasswordInput } from "@/shared/ui/password-input";
import { useEnterAccount } from "../hooks/use-enter-account";

type Props = {
  /** A conta tem senha: a API pede a atual. */
  password: boolean;
};

/** Erros que são da senha digitada (vão no campo); o resto vai embaixo do botão. */
const PASSWORD_ERRORS = new Set([
  "auth.wrong_password",
  "auth.too_many_attempts",
]);

/**
 * Agenda a exclusão da conta. Com senha, pede a atual; ao enviar, o
 * `ConfirmModal` diz o que acontece, e só o "sim" chama a API. Deu certo, a
 * sessão acabou: vai para a página de entrar, carregada do zero (o cabeçalho
 * ainda é de quem estava logado), com a data e como cancelar.
 */
export function DeleteAccountForm({ password }: Props) {
  const t = useTranslations("Account");
  const enter = useEnterAccount();
  const { run, pending, error } = useApiAction();
  const { ref, canSubmit } = useFormValidity();
  const [current, setCurrent] = useState("");
  const [confirming, setConfirming] = useState(false);
  const [passwordError, setPasswordError] = useState(false);
  // Enquanto a página de entrar carrega, o botão continua pendente.
  const [leaving, setLeaving] = useState(false);
  const opener = useRef<HTMLElement | null>(null);

  const close = () => {
    setConfirming(false);
    // O Modal sai da árvore e o foco cairia no body.
    requestAnimationFrame(() => opener.current?.focus());
  };

  const schedule = () =>
    run(
      () =>
        api.POST("/me/deletion", {
          body: password ? { current_password: current } : {},
        }),
      ({ scheduled_for }) => {
        setLeaving(true);
        enter(`/login?deletion_scheduled=${encodeURIComponent(scheduled_for)}`);
      },
      (code) => setPasswordError(PASSWORD_ERRORS.has(code)),
    );

  return (
    <Form
      ref={ref}
      onSubmit={(event) => {
        event.preventDefault();
        if (pending || leaving) return;
        // O botão ou, com Enter, o campo: o foco volta para lá ao fechar.
        opener.current = document.activeElement as HTMLElement | null;
        setConfirming(true);
      }}
    >
      <Stack gap={5}>
        {password && (
          <PasswordInput
            labelText={t("currentPassword")}
            helperText={t("deletion.passwordHint")}
            required
            autoComplete="current-password"
            value={current}
            onChange={(e) => setCurrent(e.target.value)}
            invalid={passwordError && error !== undefined}
            invalidText={error}
          />
        )}
        <div>
          <Button
            type="submit"
            kind="danger--tertiary"
            pending={pending || leaving}
            disabled={!canSubmit}
          >
            {t("deletion.submit")}
          </Button>
        </div>
        {error && !passwordError && <ErrorText>{error}</ErrorText>}
      </Stack>
      <ConfirmModal
        open={confirming}
        onClose={close}
        onConfirm={() => {
          close();
          schedule();
        }}
        title={t("deletion.title")}
        confirmLabel={t("deletion.submit")}
        destructive
      >
        {`${t("deletion.confirm")} ${t("deletion.consequences")}`}
      </ConfirmModal>
    </Form>
  );
}
