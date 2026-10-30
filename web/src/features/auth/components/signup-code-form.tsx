"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useActionState, useEffect, useId, useRef, useState } from "react";

import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import { digitsOnly } from "@/shared/lib/input-filters";
import { Button } from "@/shared/ui/button";
import { TextInput } from "@/shared/ui/text-input";
import { useEnterAccount } from "../hooks/use-enter-account";
import { takeSignupReturnTo } from "../lib/signup-return-to";

const CODE_LENGTH = 6;

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
 * O código de 6 dígitos do email de cadastro, no lugar de abrir o link: o
 * campo só aceita dígitos (colar "123 456" vira "123456") e envia sozinho ao
 * completar; o botão, travado até lá, serve para tentar de novo o mesmo. Certo, cria a conta e
 * entra; errado, o erro fica no campo, com as tentativas que restam, até o
 * código mudar.
 */
export function SignupCodeForm({ email, returnTo, focusOnMount }: Props) {
  const t = useTranslations("Signup.code");
  const tSignup = useTranslations("Signup");
  const errorMessage = useApiErrorMessage();
  const enter = useEnterAccount();
  const id = useId();
  const { ref: form, canSubmit } = useFormValidity();
  const [code, setCode] = useState("");
  // O código que já foi sozinho: o mesmo não vai de novo sem o botão.
  const sent = useRef<string>(undefined);
  // Enquanto a página da conta carrega, o botão continua pendente.
  const [leaving, setLeaving] = useState(false);

  const [result, submit, pending] = useActionState<Result, FormData>(
    async (_, data) => {
      const typed = String(data.get("code") ?? "");
      if (typed.length !== CODE_LENGTH) {
        return { error: t("incomplete"), code: typed };
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
  const error = result.code === code ? result.error : undefined;

  useEffect(() => {
    if (focusOnMount) document.getElementById(id)?.focus();
  }, [focusOnMount, id]);

  useEffect(() => {
    if (code.length < CODE_LENGTH) sent.current = undefined;
    else if (sent.current !== code) {
      sent.current = code;
      form.current?.requestSubmit();
    }
  }, [code, form]);

  return (
    <Form ref={form} action={submit}>
      <Stack gap={5}>
        <TextInput
          id={id}
          name="code"
          labelText={t("label")}
          required
          minLength={CODE_LENGTH}
          filter={digitsOnly}
          inputMode="numeric"
          autoComplete="one-time-code"
          value={code}
          // Sem `maxLength`: colar "123 456" passaria do tamanho antes do filtro.
          onChange={(e) => setCode(e.target.value.slice(0, CODE_LENGTH))}
          invalid={Boolean(error)}
          invalidText={error}
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
