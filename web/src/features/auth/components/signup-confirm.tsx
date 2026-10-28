"use client";

import { Layer, Stack, Tile } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { Button } from "@/shared/ui/button";
import { Link } from "@/shared/ui/link";
import { Notice } from "@/shared/ui/notice";
import { useEnterAccount } from "../hooks/use-enter-account";
import { useLinkToken } from "../hooks/use-link-token";
import { takeSignupReturnTo } from "../lib/signup-return-to";

/**
 * O link do email de cadastro (`#token`). Não confirma sozinho ao abrir: os
 * verificadores de link dos provedores de email abrem a página e gastariam o
 * link (e a sessão ficaria com o robô). O botão cria a conta e entra, indo
 * para onde o cadastro voltaria (`takeSignupReturnTo`). Recusado (link
 * vencido, nome ou email tomados nesse meio tempo) ou sem token, o motivo e
 * os caminhos: criar a conta de novo ou entrar.
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
              <Link href="/signup">{t("signupAgain")}</Link>
              <Link href="/login">{t("login")}</Link>
            </Stack>
          )}
        </Stack>
      </Layer>
    </Tile>
  );
}
