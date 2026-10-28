"use client";

import { Layer, Stack, Tile } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useId, useState } from "react";

import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { Link } from "@/shared/ui/link";
import { Notice } from "@/shared/ui/notice";
import { useLinkToken } from "../hooks/use-link-token";

/**
 * O link do email mandado ao trocar o email da conta (`#token`). Não exige
 * login (pode ser aberto em outro aparelho) e não confirma sozinho ao abrir:
 * os verificadores de link dos provedores de email gastariam o link. Feito,
 * o aviso (que recebe o foco, no lugar do botão) e o caminho para a conta;
 * recusado ou sem token, o motivo e o mesmo caminho, onde se pede outro link.
 */
export function EmailConfirm() {
  const t = useTranslations("Account.emailConfirm");
  const tAuth = useTranslations("Auth");
  const token = useLinkToken();
  const { run, pending, error } = useApiAction();
  const [done, setDone] = useState(false);
  const doneId = useId();

  const confirm = () => {
    if (!token) return;
    run(
      () => api.POST("/auth/email/confirm", { body: { token } }),
      () => {
        setDone(true);
        focusIfLost(doneId);
      },
    );
  };
  const problem = token === "" ? tAuth("linkIncomplete") : error;

  return (
    <Tile>
      <Layer>
        {done ? (
          <div id={doneId} tabIndex={-1}>
            <Stack gap={5}>
              <Notice kind="success" title={t("doneTitle")}>
                {t("done")}
              </Notice>
              <Link href="/account/logins">{t("goToAccount")}</Link>
            </Stack>
          </div>
        ) : (
          <Stack gap={6}>
            {token !== "" && (
              <Button
                pending={pending || token === undefined}
                onClick={confirm}
              >
                {t("submit")}
              </Button>
            )}
            {problem && (
              <Stack gap={5}>
                <Notice kind="error">{problem}</Notice>
                <Link href="/account/logins">{t("goToAccount")}</Link>
              </Stack>
            )}
          </Stack>
        )}
      </Layer>
    </Tile>
  );
}
