"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useActionState, useState } from "react";

import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import { Button } from "@/shared/ui/button";
import { useCodeAutoSubmit } from "../hooks/use-code-auto-submit";
import { useEnterAccount } from "../hooks/use-enter-account";
import { takeSignupReturnTo } from "../lib/signup-return-to";
import { CODE_LENGTH, CodeInput } from "./code-input";

type Props = {
  /** O email do cadastro, na forma que a API devolveu. */
  email: string;
  /** Para onde ir depois de criar a conta. */
  returnTo: string;
  /** Leva o foco ao campo ao aparecer (a tela "confira seu email" nova). */
  focusOnMount: boolean;
};

/** O erro do último envio e o código que o causou. */
type Result = { error?: string; code?: string };

/**
 * O código de 6 dígitos do email de cadastro, no lugar de abrir o link: envia
 * sozinho ao completar; o botão, travado até lá, serve para tentar de novo o
 * mesmo. Certo, cria a conta e entra; errado, o erro fica no campo até o código
 * mudar.
 */
export function SignupCodeForm({ email, returnTo, focusOnMount }: Props) {
  const tCode = useTranslations("Auth.code");
  const tSignup = useTranslations("Signup");
  const errorMessage = useApiErrorMessage();
  const enter = useEnterAccount();
  const { ref: form, canSubmit } = useFormValidity();
  const [code, setCode] = useState("");
  // Enquanto a página da conta carrega, o botão continua pendente.
  const [leaving, setLeaving] = useState(false);

  const [result, submit, pending] = useActionState<Result, FormData>(
    async (_, data) => {
      const typed = String(data.get("code") ?? "");
      if (typed.length !== CODE_LENGTH) {
        return { error: tCode("incomplete"), code: typed };
      }
      const outcome = await callApi(
        api.POST("/auth/signup/code", { body: { email, code: typed } }),
      );
      if (!outcome.ok) {
        return {
          error: errorMessage(outcome.code, outcome.params),
          code: typed,
        };
      }
      takeSignupReturnTo();
      setLeaving(true);
      enter(returnTo);
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
          focusOnMount={focusOnMount}
        />
        <Button
          type="submit"
          disabled={!canSubmit}
          pending={pending || leaving}
        >
          {tSignup("submit")}
        </Button>
      </Stack>
    </Form>
  );
}
