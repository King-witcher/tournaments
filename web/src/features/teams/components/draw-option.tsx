"use client";

import { useTranslations } from "next-intl";

import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { TEAMS_ID } from "../lib/focus";
import type { Formation } from "../lib/formation";
import styles from "./draw-option.module.scss";

type Props = {
  tenant: string;
  tournament: string;
  method: "random" | "balanced";
  /** Já há times: o sorteio desfaz todos, então pergunta antes. */
  ask: boolean;
  /** Sem gente para um time (o motivo fica na seção). */
  disabled: boolean;
  onDrawn: (formation: Formation) => void;
};

/**
 * Um jeito de sortear os times: o botão (com a pergunta, se já há times), o
 * erro embaixo dele e a frase do que faz. Depois, o foco vai para a lista
 * dos times (o botão troca de forma quando os primeiros times aparecem).
 */
export function DrawOption({
  tenant,
  tournament,
  method,
  ask,
  disabled,
  onDrawn,
}: Props) {
  const t = useTranslations("Team.draw");
  const { run, pending, error } = useApiAction();
  const label = t(method);
  const draw = () =>
    run(
      () =>
        api.POST("/tenants/{slug}/tournaments/{tournament}/teams/draw", {
          params: { path: { slug: tenant, tournament } },
          body: { method },
        }),
      (data) => {
        onDrawn(data);
        focusIfLost(TEAMS_ID);
      },
    );

  return (
    <li className={styles.option}>
      <ActionWithNote error={error}>
        {ask ? (
          <ConfirmButton
            destructive={false}
            kind="tertiary"
            size="md"
            pending={pending}
            disabled={disabled}
            question={t("confirm")}
            onConfirm={draw}
          >
            {label}
          </ConfirmButton>
        ) : (
          <Button
            kind="tertiary"
            pending={pending}
            disabled={disabled}
            onClick={draw}
          >
            {label}
          </Button>
        )}
      </ActionWithNote>
      <p className={styles.hint}>
        {t(method === "random" ? "randomHint" : "balancedHint")}
      </p>
    </li>
  );
}
