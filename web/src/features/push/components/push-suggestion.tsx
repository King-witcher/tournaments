"use client";

import { useTranslations } from "next-intl";
import { useEffect, useId, useState } from "react";

import { MY_PARTICIPATION_ID } from "@/features/participants/lib/focus";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import { Link } from "@/shared/ui/link";
import { usePush } from "../hooks/use-push";
import {
  dismissSuggestion,
  suggestionDismissed,
} from "../lib/suggestion-dismissed";
import styles from "./push-suggestion.module.scss";

/**
 * A sugestão de ativar as notificações com o site fechado, num bloco à parte
 * abaixo de "Sua situação", para quem joga um torneio que não acabou. Só
 * aparece enquanto a permissão não foi pedida neste navegador e a pessoa não
 * a dispensou ("Agora não", lembrado neste navegador); no iPhone fora do site
 * instalado, no lugar do botão, como instalar. Depois do clique, o resultado
 * (ativado, ou bloqueado e onde mudar).
 */
export function PushSuggestion() {
  const t = useTranslations("Notifications.push");
  const { status, asked, pending, error, enable } = usePush();
  const [acted, setActed] = useState(false);
  // Escondida até ler o armazenamento, para não piscar para quem já dispensou.
  const [dismissed, setDismissed] = useState(true);
  const resultId = useId();
  const answered = acted && (status === "on" || status === "denied");

  useEffect(() => {
    setDismissed(suggestionDismissed());
  }, []);

  // O botão some com a resposta (que pode demorar: a pessoa lê a pergunta
  // do navegador): o foco vai para o resultado.
  useEffect(() => {
    if (answered) focusIfLost(resultId);
  }, [answered, resultId]);

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
  if (dismissed) return null;
  if (status === "install") {
    return <p className={styles.hint}>{t("promptInstall")}</p>;
  }
  if (status !== "off" || (asked && !acted)) return null;

  return (
    <div className={styles.suggestion}>
      <p>{t("prompt")}</p>
      <ActionWithNote error={error}>
        <div className={styles.actions}>
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
          <Button
            kind="ghost"
            size="sm"
            disabled={pending}
            onClick={() => {
              dismissSuggestion();
              setDismissed(true);
              focusIfLost(MY_PARTICIPATION_ID);
            }}
          >
            {t("dismiss")}
          </Button>
        </div>
      </ActionWithNote>
    </div>
  );
}
