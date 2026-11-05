"use client";

import { useTranslations } from "next-intl";
import { useActionState, useId, useState } from "react";

import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFocusInvalid } from "@/shared/hooks/use-focus-invalid";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import { type FormResult, formError } from "@/shared/lib/form-result";
import { Button } from "@/shared/ui/button";
import { Form } from "@/shared/ui/carbon/form";
import { Layer } from "@/shared/ui/carbon/layer";
import { Stack } from "@/shared/ui/carbon/stack";
import { Tile } from "@/shared/ui/carbon/tile";
import { Link } from "@/shared/ui/link";
import { Notice } from "@/shared/ui/notice";
import { useEnterAccount } from "../hooks/use-enter-account";
import { useLinkToken } from "../hooks/use-link-token";
import { useNewPassword } from "../hooks/use-new-password";
import { afterEmailLogin } from "../lib/after-email-login";
import { type AuthField, errorField } from "../lib/auth-field";
import { NewPasswordFields } from "./new-password-fields";

/**
 * O link do email de entrar (`#token`): um botão "Entrar" e a senha nova
 * opcional. Não entra sozinho ao abrir: os verificadores de link dos provedores
 * de email abrem a página e às vezes rodam o script, e gastariam o link. Senha
 * fora do formato não gasta o link; recusado ou sem token, o motivo e o caminho
 * para pedir outro.
 */
export function EmailLoginConfirm() {
  const t = useTranslations("Login.email");
  const tAuth = useTranslations("Auth");
  const errorMessage = useApiErrorMessage();
  const token = useLinkToken();
  const enter = useEnterAccount();
  const prefix = useId();
  const { ref, canSubmit } = useFormValidity();
  const newPassword = useNewPassword();
  // Enquanto a página da conta carrega, o botão continua pendente.
  const [leaving, setLeaving] = useState(false);

  const [result, submit, pending] = useActionState<FormResult<AuthField>>(
    async () => {
      if (!token) return {};
      if (newPassword.differs) {
        return { error: tAuth("passwordsDiffer"), field: "confirm" };
      }
      const outcome = await callApi(
        api.POST("/auth/email/login/confirm", {
          body: { token, password: newPassword.value },
        }),
      );
      if (!outcome.ok) {
        return {
          error: errorMessage(outcome.code, outcome.params),
          field: errorField(outcome.code),
        };
      }
      setLeaving(true);
      enter(
        afterEmailLogin(
          outcome.data.deletion_cancelled,
          newPassword.value !== undefined,
          "/",
        ),
      );
      return {};
    },
    {},
  );
  useFocusInvalid(prefix, result);
  const problem = (message: string) => (
    <Notice
      kind="error"
      action={<Link href="/login/email">{t("requestAgain")}</Link>}
    >
      {message}
    </Notice>
  );
  const general = formError(result);

  return (
    <Tile>
      <Layer>
        {token === "" ? (
          problem(tAuth("linkIncomplete"))
        ) : (
          <Form ref={ref} action={submit}>
            <Stack gap={6}>
              <NewPasswordFields
                state={newPassword}
                prefix={prefix}
                result={result}
              />
              {general && problem(general)}
              <Button
                type="submit"
                pending={pending || leaving || token === undefined}
                disabled={!canSubmit}
              >
                {tAuth("submit")}
              </Button>
            </Stack>
          </Form>
        )}
      </Layer>
    </Tile>
  );
}
