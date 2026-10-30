"use client";

import { InlineLoading, Layer, Stack, Tile } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useEffect, useRef } from "react";

import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { Link } from "@/shared/ui/link";
import { Notice } from "@/shared/ui/notice";
import { useEnterAccount } from "../hooks/use-enter-account";
import { useLinkToken } from "../hooks/use-link-token";
import { takeSignupReturnTo } from "../lib/signup-return-to";

/**
 * O link do email de cadastro (`#token`): ao carregar no navegador, cria a
 * conta e entra, indo para onde o cadastro voltaria (`takeSignupReturnTo`),
 * sem clique. A confirmação é um POST do script da página: o token vem
 * depois do `#`, que não vai ao servidor, então o verificador de links do
 * provedor de email que só baixa a página (GET) não gasta o link. Recusado
 * (link vencido ou já usado, nome ou email tomados nesse meio tempo) ou sem
 * token, o motivo e os caminhos: criar conta ou entrar.
 */
export function SignupConfirm() {
  const t = useTranslations("Signup.confirm");
  const tAuth = useTranslations("Auth");
  const token = useLinkToken();
  const enter = useEnterAccount();
  const { run, error } = useApiAction();
  // Uma vez só, também no efeito dobrado do modo estrito: o link vale uma vez.
  const started = useRef(false);

  useEffect(() => {
    if (!token || started.current) return;
    started.current = true;
    run(
      () => api.POST("/auth/signup/confirm", { body: { token } }),
      () => enter(takeSignupReturnTo()),
    );
  }, [token, run, enter]);

  const problem = token === "" ? tAuth("linkIncomplete") : error;

  return (
    <Tile>
      <Layer>
        {problem ? (
          <Stack gap={5}>
            <Notice kind="error">{problem}</Notice>
            <Link href="/signup">{t("signup")}</Link>
            <Link href="/login">{t("login")}</Link>
          </Stack>
        ) : (
          // Até a página da conta abrir (ou o erro chegar).
          <InlineLoading status="active" description={t("creating")} />
        )}
      </Layer>
    </Tile>
  );
}
