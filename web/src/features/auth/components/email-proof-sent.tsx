"use client";

import { useTranslations } from "next-intl";
import { useActionState, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import { Form } from "@/shared/ui/carbon/form";
import { Stack } from "@/shared/ui/carbon/stack";
import { Notice } from "@/shared/ui/notice";
import { useCodeAutoSubmit } from "../hooks/use-code-auto-submit";
import { useCooldown } from "../hooks/use-cooldown";
import { LOGIN_SECTION } from "../lib/login-sections";
import { CODE_LENGTH, CodeInput } from "./code-input";

/** A espera da API entre dois envios ao mesmo endereço, que não dá erro. */
const RESEND_AFTER_SECS = 60;

type Props = {
  email: string;
  pending: boolean;
  /** O erro do último reenvio, já traduzido. */
  error?: string;
  onResend: () => void;
  focusOnMount: boolean;
};

/** O erro do último envio e o código que o causou. */
type Result = { error?: string; code?: string };

/**
 * Mandado o código da prova: o campo, que envia sozinho ao completar, e reenviar,
 * que espera o tempo da API. Certo, esta sessão pode mudar os logins: a página
 * recarrega e o foco vai para o bloco do email.
 */
export function EmailProofSent({
  email,
  pending,
  error,
  onResend,
  focusOnMount,
}: Props) {
  const t = useTranslations("Account.email");
  const tLogin = useTranslations("Login.email");
  const tCode = useTranslations("Auth.code");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const { ref: form, canSubmit } = useFormValidity();
  const cooldown = useCooldown(RESEND_AFTER_SECS);
  const [code, setCode] = useState("");

  const [result, submit, submitting] = useActionState<Result, FormData>(
    async (_, data) => {
      const typed = String(data.get("code") ?? "");
      if (typed.length !== CODE_LENGTH) {
        return { error: tCode("incomplete"), code: typed };
      }
      const outcome = await callApi(
        api.POST("/auth/email/proof/code", { body: { code: typed } }),
      );
      if (!outcome.ok) {
        return {
          error: errorMessage(outcome.code, outcome.params),
          code: typed,
        };
      }
      router.refresh();
      focusIfLost(LOGIN_SECTION.email);
      return {};
    },
    {},
  );
  useCodeAutoSubmit(code, form);

  return (
    <Stack gap={6}>
      <Notice kind="success" title={t("sentTitle")}>
        {t("proofSent", { email })}
      </Notice>
      <Form ref={form} action={submit}>
        <Stack gap={5}>
          <CodeInput
            value={code}
            onChange={setCode}
            error={result.code === code ? result.error : undefined}
            focusOnMount={focusOnMount}
          />
          <Button type="submit" disabled={!canSubmit} pending={submitting}>
            {t("proofSubmit")}
          </Button>
        </Stack>
      </Form>
      <ActionWithNote error={error}>
        <Button
          kind="tertiary"
          pending={pending}
          disabled={cooldown.left > 0}
          onClick={onResend}
        >
          {cooldown.left > 0
            ? tLogin("resendIn", { seconds: cooldown.left })
            : tLogin("resend")}
        </Button>
      </ActionWithNote>
    </Stack>
  );
}
