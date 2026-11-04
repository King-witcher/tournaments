"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useActionState, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { useCodeAutoSubmit } from "../hooks/use-code-auto-submit";
import { LOGIN_SECTION } from "../lib/login-sections";
import { CODE_LENGTH, CodeInput } from "./code-input";

type Props = {
  /** O email da conta mudou; o refresh remonta o formulário do email. */
  onConfirmed: () => void;
};

/** O erro do último envio e o código que o causou. */
type Result = { error?: string; code?: string };

/**
 * O código do email de troca, no lugar de abrir o link (há navegadores em que
 * não dá para ir até o link): envia sozinho ao completar, e o erro fica no campo
 * até o código mudar. Certo, o email muda, a página recarrega e o foco vai para
 * o bloco do email.
 */
export function EmailChangeCodeForm({ onConfirmed }: Props) {
  const t = useTranslations("Account.emailConfirm");
  const tCode = useTranslations("Auth.code");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const { ref: form, canSubmit } = useFormValidity();
  const [code, setCode] = useState("");

  const [result, submit, pending] = useActionState<Result, FormData>(
    async (_, data) => {
      const typed = String(data.get("code") ?? "");
      if (typed.length !== CODE_LENGTH) {
        return { error: tCode("incomplete"), code: typed };
      }
      const outcome = await callApi(
        api.POST("/auth/email/code", { body: { code: typed } }),
      );
      if (!outcome.ok) {
        return {
          error: errorMessage(outcome.code, outcome.params),
          code: typed,
        };
      }
      onConfirmed();
      router.refresh();
      focusIfLost(LOGIN_SECTION.email);
      return {};
    },
    {},
  );
  useCodeAutoSubmit(code, form);

  return (
    <Form ref={form} action={submit}>
      <Stack gap={5}>
        <CodeInput
          value={code}
          onChange={setCode}
          error={result.code === code ? result.error : undefined}
          focusOnMount
        />
        <Button type="submit" disabled={!canSubmit} pending={pending}>
          {t("submit")}
        </Button>
      </Stack>
    </Form>
  );
}
