"use client";

import { useTranslations } from "next-intl";
import { type RefObject, useRef, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { FormModal } from "@/shared/ui/form-modal";
import { PasswordInput } from "@/shared/ui/password-input";
import { LOGIN_SECTION } from "../lib/login-sections";

/** Erros que são da senha digitada (vão no campo); o resto vai acima dos botões. */
const PASSWORD_ERRORS = new Set([
  "auth.wrong_password",
  "auth.too_many_attempts",
]);

/**
 * "Remover senha", ao lado de "Trocar senha": abre o `FormModal` que diz o
 * que muda e pede a senha atual (a API confere). A senha sempre pode sair: a
 * conta que a tem tem email, que entra sozinho. Removida, a página recarrega e o
 * foco vai para o bloco da senha (o botão some).
 */
export function RemovePasswordButton() {
  const t = useTranslations("Account.password");
  const router = useRouter();
  const trigger = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(false);

  return (
    <>
      <Button
        ref={trigger}
        kind="danger--tertiary"
        onClick={() => setOpen(true)}
      >
        {t("remove")}
      </Button>
      {open && (
        <RemovePasswordModal
          trigger={trigger}
          onClose={() => setOpen(false)}
          onDone={() => {
            setOpen(false);
            router.refresh();
            focusIfLost(LOGIN_SECTION.password);
          }}
        />
      )}
    </>
  );
}

/** O Modal aberto: um a cada abertura, para começar vazio. */
function RemovePasswordModal({
  trigger,
  onClose,
  onDone,
}: {
  trigger: RefObject<HTMLButtonElement | null>;
  onClose: () => void;
  onDone: () => void;
}) {
  const t = useTranslations("Account");
  const { run, pending, error } = useApiAction();
  const [current, setCurrent] = useState("");
  const [passwordError, setPasswordError] = useState(false);

  return (
    <FormModal
      title={t("password.remove")}
      submitLabel={t("password.remove")}
      destructive
      pending={pending}
      submitDisabled={current === ""}
      error={passwordError ? undefined : error}
      returnFocusTo={trigger}
      onClose={onClose}
      onSubmit={() =>
        run(
          () =>
            api.DELETE("/auth/password", {
              body: { current_password: current },
            }),
          onDone,
          (code) => setPasswordError(PASSWORD_ERRORS.has(code)),
        )
      }
    >
      <p>{t("password.removeConfirm")}</p>
      <PasswordInput
        labelText={t("currentPassword")}
        required
        autoComplete="current-password"
        value={current}
        onChange={(e) => setCurrent(e.target.value)}
        invalid={passwordError && error !== undefined}
        invalidText={error}
      />
    </FormModal>
  );
}
