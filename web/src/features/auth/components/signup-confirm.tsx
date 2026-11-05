"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { Button } from "@/shared/ui/button";
import { Layer } from "@/shared/ui/carbon/layer";
import { Stack } from "@/shared/ui/carbon/stack";
import { Tile } from "@/shared/ui/carbon/tile";
import { Link } from "@/shared/ui/link";
import { Notice } from "@/shared/ui/notice";
import { useEnterAccount } from "../hooks/use-enter-account";
import { useLinkToken } from "../hooks/use-link-token";
import { takeSignupReturnTo } from "../lib/signup-return-to";

/**
 * O link do email de cadastro (`#token`): um botão "Confirmar" cria a conta e
 * entra, indo para onde o cadastro voltaria (`takeSignupReturnTo`). Não confirma
 * sozinho ao abrir: um verificador de links que roda o script da página criaria
 * a conta que um atacante pediu com o email da vítima e a senha dele, e a vítima
 * depois entraria nela por email. Recusado (link vencido ou já usado, email
 * tomado nesse meio tempo) ou sem token, o motivo e os caminhos: criar conta ou
 * entrar.
 */
export function SignupConfirm() {
  const t = useTranslations("Signup.confirm");
  const tAuth = useTranslations("Auth");
  const token = useLinkToken();
  const enter = useEnterAccount();
  const { run, pending, error } = useApiAction();
  // Enquanto a página da conta carrega, o botão continua pendente.
  const [leaving, setLeaving] = useState(false);

  const confirm = () => {
    if (!token) return;
    run(
      () => api.POST("/auth/signup/confirm", { body: { token } }),
      () => {
        setLeaving(true);
        enter(takeSignupReturnTo());
      },
    );
  };
  const problem = token === "" ? tAuth("linkIncomplete") : error;

  return (
    <Tile>
      <Layer>
        <Stack gap={6}>
          {token !== "" && (
            <Button
              pending={pending || leaving || token === undefined}
              onClick={confirm}
            >
              {t("submit")}
            </Button>
          )}
          {problem && (
            <Stack gap={5}>
              <Notice kind="error">{problem}</Notice>
              <Link href="/signup">{t("signup")}</Link>
              <Link href="/login">{t("login")}</Link>
            </Stack>
          )}
        </Stack>
      </Layer>
    </Tile>
  );
}
