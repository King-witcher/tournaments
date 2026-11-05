"use client";

import { useTranslations } from "next-intl";
import { useActionState, useEffect, useId, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFocusInvalid } from "@/shared/hooks/use-focus-invalid";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import { focusIfLost } from "@/shared/lib/focus";
import {
  type FormResult,
  fieldId,
  formError,
  invalidProps,
} from "@/shared/lib/form-result";
import { Button } from "@/shared/ui/button";
import { Form } from "@/shared/ui/carbon/form";
import { Stack } from "@/shared/ui/carbon/stack";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { Notice } from "@/shared/ui/notice";
import { PasswordInput } from "@/shared/ui/password-input";
import { TextInput } from "@/shared/ui/text-input";
import { type AuthField, errorField } from "../lib/auth-field";
import { LOGIN_SECTION } from "../lib/login-sections";
import { EmailChangeCodeForm } from "./email-change-code-form";
import { RemoveEmailButton } from "./remove-email-button";

type Props = {
  /** O email confirmado de agora, se houver: vem no campo, para editar. */
  current?: string;
  /** A conta tem senha: a API pede a atual para trocar o email, e ele não sai. */
  hasPassword: boolean;
  /** O Discord ou o Google está vinculado: sem o email, a conta ainda entra. */
  providerLinked: boolean;
  /** O código confirmou o email novo. */
  onConfirmed: () => void;
};

/**
 * Para onde foram o link e o código, na forma canônica que a API devolveu, e o
 * email que foi enviado, para o aviso e o erro sumirem quando o campo muda
 * (inclusive pelo "Cancelar").
 */
type Result = FormResult<AuthField> & { sentTo?: string; typed?: string };

/**
 * Cadastra, troca ou remove o email da conta. O campo vem com o de agora; a
 * API manda um link e um código ao endereço novo, e o email só muda com um dos
 * dois: o link abre `/email/confirm`, e o código entra aqui mesmo, no campo que
 * aparece depois do envio, com "Pedir outro email" de volta ao formulário. Com
 * senha na conta, a atual aparece quando o campo muda, até o envio. "Cancelar"
 * volta o campo ao de agora. O erro de um campo (email inválido, igual ao de
 * agora ou de outra conta, senha errada) vai nele, que recebe o foco. Chegando
 * pelo aviso de login de reserva (`#login-email`), o foco já vem no campo.
 */
export function ChangeEmailForm({
  current,
  hasPassword,
  providerLinked,
  onConfirmed,
}: Props) {
  const t = useTranslations("Account.email");
  const tAccount = useTranslations("Account");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const prefix = useId();
  const { ref, canSubmit } = useFormValidity();
  const [email, setEmail] = useState(current ?? "");
  const [password, setPassword] = useState("");
  // O envio que "Pedir outro email" dispensou: o seguinte traz o aviso de volta.
  const [dismissed, setDismissed] = useState<Result>();
  // O email gravado é minúsculo: mudar só a caixa não é trocar.
  const changed = email.trim().toLowerCase() !== (current ?? "");
  const askPassword = hasPassword && changed;

  useEffect(() => {
    if (window.location.hash === `#${LOGIN_SECTION.email}`) {
      document.getElementById(fieldId(prefix, "email"))?.focus();
    }
  }, [prefix]);

  const [result, submit, pending] = useActionState<Result>(async () => {
    if (!changed) {
      return {
        error: errorMessage("auth.email_unchanged"),
        field: "email",
        typed: email,
      };
    }
    const outcome = await callApi(
      api.PUT("/auth/email", {
        body: {
          email,
          current_password: askPassword ? password : undefined,
        },
      }),
    );
    if (!outcome.ok) {
      // A prova venceu com a tela aberta: a página volta a pedir o código.
      if (outcome.code === "auth.email_proof_required") router.refresh();
      return {
        error: errorMessage(outcome.code, outcome.params),
        field: errorField(outcome.code),
        typed: email,
      };
    }
    setPassword("");
    return { sentTo: outcome.data.email, typed: email };
  }, {});
  useFocusInvalid(prefix, result);
  // O resultado vale para o email enviado; com o campo mudado, sai.
  const shown: Result = !pending && result.typed === email ? result : {};
  const general = formError(shown);
  const sent = result === dismissed ? undefined : shown.sentTo;
  const editing = Boolean(current) && changed && !sent;
  const removeBlocked = hasPassword
    ? t("removeBlocked")
    : !providerLinked
      ? t("removeLastLogin")
      : undefined;

  return (
    <Stack gap={6}>
      <Form ref={ref} action={submit}>
        <Stack gap={6}>
          <TextInput
            id={fieldId(prefix, "email")}
            type="email"
            labelText={t("label")}
            required
            autoComplete="email"
            maxLength={254}
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            {...invalidProps(shown, "email")}
          />
          {askPassword && !sent && (
            <PasswordInput
              id={fieldId(prefix, "current")}
              labelText={tAccount("currentPassword")}
              required
              autoComplete="current-password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              {...invalidProps(shown, "current")}
            />
          )}
          {general && <FormError>{general}</FormError>}
          <FormActions
            submitLabel={current ? t("change") : t("add")}
            pending={pending}
            disabled={!canSubmit || Boolean(sent)}
            onCancel={
              editing
                ? () => {
                    setEmail(current ?? "");
                    setPassword("");
                  }
                : undefined
            }
            extra={current && <RemoveEmailButton blocked={removeBlocked} />}
          />
        </Stack>
      </Form>
      {sent && (
        <>
          <Notice kind="success" title={t("sentTitle")}>
            {t("sent", { email: sent })}
          </Notice>
          <EmailChangeCodeForm onConfirmed={onConfirmed} />
          <Button
            kind="ghost"
            onClick={() => {
              setDismissed(result);
              focusIfLost(fieldId(prefix, askPassword ? "current" : "email"));
            }}
          >
            {t("requestAgain")}
          </Button>
        </>
      )}
    </Stack>
  );
}
