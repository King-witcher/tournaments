"use client";

import { Form, Layer, Stack, Tile } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useActionState, useId, useState } from "react";

import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFocusInvalid } from "@/shared/hooks/use-focus-invalid";
import {
  type FormResult,
  fieldId,
  formError,
  invalidProps,
} from "@/shared/lib/form-result";
import { Button } from "@/shared/ui/button";
import { Link } from "@/shared/ui/link";
import { Notice } from "@/shared/ui/notice";
import { PasswordInput } from "@/shared/ui/password-input";
import { useEnterAccount } from "../hooks/use-enter-account";
import { useLinkToken } from "../hooks/use-link-token";
import { type AuthField, errorField } from "../lib/auth-field";
import { DELETION_CANCELLED } from "../lib/deletion-cancelled";

/** Para onde vai quem trocou a senha: a aba Logins, com o aviso de feito. */
const AFTER_RESET = "/account/logins?password_reset=1";

/**
 * O link do email de recuperação (`#token`): a senha nova e a repetida. Salvar
 * troca a senha, encerra todas as sessões da conta e entra numa nova, indo
 * para a aba Logins com o aviso (`PasswordResetDone`); se a exclusão da
 * conta estava agendada, entrar a cancela, e vai para a conta, que avisa.
 * Senha fora do formato não gasta o link (o erro vai no campo); link vencido
 * ou sem token, o motivo e o caminho para pedir outro.
 */
export function PasswordResetForm() {
  const t = useTranslations("Login.reset");
  const tAuth = useTranslations("Auth");
  const tAccount = useTranslations("Account.password");
  const errorMessage = useApiErrorMessage();
  const token = useLinkToken();
  const enter = useEnterAccount();
  const prefix = useId();
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  // Enquanto a página da conta carrega, o botão continua pendente.
  const [leaving, setLeaving] = useState(false);

  const [result, submit, pending] = useActionState<FormResult<AuthField>>(
    async () => {
      if (!token) return {};
      if (password !== confirm) {
        return { error: tAuth("passwordsDiffer"), field: "confirm" };
      }
      const outcome = await callApi(
        api.POST("/auth/password/reset/confirm", {
          body: { token, password },
        }),
      );
      if (!outcome.ok) {
        return {
          error: errorMessage(outcome.code, outcome.params),
          field: errorField(outcome.code),
        };
      }
      setLeaving(true);
      enter(outcome.data.deletion_cancelled ? DELETION_CANCELLED : AFTER_RESET);
      return {};
    },
    {},
  );
  useFocusInvalid(prefix, result);
  const problem = (message: string) => (
    <Notice
      kind="error"
      action={<Link href="/login/forgot">{t("requestAgain")}</Link>}
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
          <Form action={submit}>
            <Stack gap={6}>
              <PasswordInput
                id={fieldId(prefix, "password")}
                labelText={tAccount("newPassword")}
                required
                autoComplete="new-password"
                minLength={8}
                maxLength={128}
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                {...invalidProps(result, "password")}
              />
              <PasswordInput
                id={fieldId(prefix, "confirm")}
                labelText={tAuth("confirmPassword")}
                required
                autoComplete="new-password"
                value={confirm}
                onChange={(e) => setConfirm(e.target.value)}
                {...invalidProps(result, "confirm")}
              />
              {general && problem(general)}
              <Button
                type="submit"
                pending={pending || leaving || token === undefined}
              >
                {t("submit")}
              </Button>
            </Stack>
          </Form>
        )}
      </Layer>
    </Tile>
  );
}
