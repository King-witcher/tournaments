"use client";

import { useTranslations } from "next-intl";
import { useEffect, useId, useState } from "react";

import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import { Link } from "@/shared/ui/link";
import { usePush } from "../hooks/use-push";
import styles from "./push-prompt.module.scss";

/**
 * O convite a ativar os avisos em "Sua situação", para quem joga um torneio
 * que não acabou. Só aparece enquanto a permissão não foi pedida neste
 * navegador (e ele não está inscrito, e o servidor tem push); no iPhone fora
 * do site instalado, no lugar do botão, como instalar. Depois do clique, o
 * resultado (ativado, ou bloqueado e onde mudar).
 */
export function PushPrompt() {
  const t = useTranslations("Notifications.push");
  const { status, asked, pending, error, enable } = usePush();
  const [acted, setActed] = useState(false);
  const resultId = useId();
  const answered = acted && (status === "on" || status === "denied");

  // O botão some com a resposta (que pode demorar: a pessoa lê a pergunta
  // do navegador): o foco vai para o resultado.
  useEffect(() => {
    if (answered) focusIfLost(resultId);
  }, [answered, resultId]);

  if (status === "install") {
    return <p className={styles.hint}>{t("promptInstall")}</p>;
  }
  if (answered) {
    return (
      <p id={resultId} tabIndex={-1} className={styles.hint}>
        {status === "on" ? (
          t("enabled")
        ) : (
          <>
            {t("blocked")}{" "}
            <Link href="/account/notifications" inline>
              {t("settings")}
            </Link>
          </>
        )}
      </p>
    );
  }
  if (status !== "off" || (asked && !acted)) return null;

  return (
    <div className={styles.prompt}>
      <p>{t("prompt")}</p>
      <ActionWithNote error={error}>
        <Button
          kind="tertiary"
          size="sm"
          pending={pending}
          onClick={() => {
            setActed(true);
            enable();
          }}
        >
          {t("enable")}
        </Button>
      </ActionWithNote>
    </div>
  );
}
