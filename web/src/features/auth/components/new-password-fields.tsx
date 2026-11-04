"use client";

import { Accordion, AccordionItem, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";

import {
  type FormResult,
  fieldId,
  invalidProps,
} from "@/shared/lib/form-result";
import { PasswordInput } from "@/shared/ui/password-input";
import type { useNewPassword } from "../hooks/use-new-password";
import type { AuthField } from "../lib/auth-field";

type Props = {
  state: ReturnType<typeof useNewPassword>;
  /** O `useId` do formulário, para os campos receberem o foco no erro. */
  prefix: string;
  result: FormResult<AuthField>;
};

/**
 * A senha nova ao entrar por email, recolhida: a maioria quer só entrar, e quem
 * veio por ter esquecido a senha abre aqui. Fechada, os campos saem do
 * formulário e não travam o envio.
 */
export function NewPasswordFields({ state, prefix, result }: Props) {
  const t = useTranslations("Login.email");
  const tAuth = useTranslations("Auth");
  const tAccount = useTranslations("Account.password");

  return (
    <Accordion>
      <AccordionItem
        title={t("newPassword")}
        open={state.open}
        onHeadingClick={({ isOpen }) => state.setOpen(isOpen)}
      >
        {state.open && (
          <Stack gap={6}>
            <PasswordInput
              id={fieldId(prefix, "password")}
              labelText={tAccount("newPassword")}
              helperText={t("newPasswordHint")}
              autoComplete="new-password"
              minLength={8}
              maxLength={128}
              value={state.password}
              onChange={(e) => state.setPassword(e.target.value)}
              {...invalidProps(result, "password")}
            />
            <PasswordInput
              id={fieldId(prefix, "confirm")}
              labelText={tAuth("confirmPassword")}
              required={state.password !== ""}
              sameAs={state.password}
              autoComplete="new-password"
              value={state.confirm}
              onChange={(e) => state.setConfirm(e.target.value)}
              {...invalidProps(result, "confirm")}
            />
          </Stack>
        )}
      </AccordionItem>
    </Accordion>
  );
}
