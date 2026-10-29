"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { ErrorText } from "@/shared/ui/error-text";
import { PasswordInput } from "@/shared/ui/password-input";
import { useOAuthStart } from "../hooks/use-oauth-start";
import type { Provider } from "../lib/providers";
import { ProviderButton } from "./provider-button";

type Props = {
  provider: Provider;
  /** Caminho, sem idioma, para onde voltar depois. */
  returnTo: string;
};

/** Erros que são da senha digitada (vão no campo); o resto vai embaixo do botão. */
const PASSWORD_ERRORS = new Set([
  "auth.wrong_password",
  "auth.too_many_attempts",
]);

/**
 * Vincula o provedor numa conta com senha: a API confere a atual antes de
 * mandar ao provedor, para quem pegou só a sessão não pôr o próprio login na
 * conta. O resto é o fluxo do `OAuthButton`.
 */
export function LinkProviderForm({ provider, returnTo }: Props) {
  const t = useTranslations("Account");
  const { start, pending, error } = useOAuthStart(provider, "link", returnTo);
  const [current, setCurrent] = useState("");
  const [passwordError, setPasswordError] = useState(false);

  return (
    <Form
      onSubmit={(event) => {
        event.preventDefault();
        if (pending) return;
        start(current, (code) => setPasswordError(PASSWORD_ERRORS.has(code)));
      }}
    >
      <Stack gap={5}>
        <PasswordInput
          labelText={t("currentPassword")}
          helperText={t(`${provider}.linkHint`)}
          required
          autoComplete="current-password"
          value={current}
          onChange={(e) => setCurrent(e.target.value)}
          invalid={passwordError && error !== undefined}
          invalidText={error}
        />
        <ProviderButton provider={provider} type="submit" pending={pending}>
          {t(`${provider}.link`)}
        </ProviderButton>
        {error && !passwordError && <ErrorText>{error}</ErrorText>}
      </Stack>
    </Form>
  );
}
