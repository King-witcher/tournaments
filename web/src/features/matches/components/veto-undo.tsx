"use client";

import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { Button } from "@/shared/ui/button";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { ErrorText } from "@/shared/ui/error-text";
import { useVetoAction } from "../hooks/use-veto-action";
import type { MatchTarget } from "../lib/match-target";
import styles from "./veto-turn.module.scss";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  veto: Schemas["VetoView"];
  names: { a: string; b: string };
  mine: Schemas["Side"] | null;
  canManage: boolean;
};

/**
 * Desfazer o último passo do jogo em andamento: o jogador pede e o
 * adversário aceita ou recusa (um passo novo cancela o pedido); a
 * organização desfaz direto (inclusive o passo sorteado pelo fim do tempo).
 */
export function VetoUndo({ target, veto, names, mine, canManage }: Props) {
  const t = useTranslations("Match.veto.undo");
  const action = useVetoAction(target);
  const request = veto.undo_request;
  if (!veto.can_undo || (!mine && !canManage)) return null;

  let player: ReactNode = null;
  if (mine && request === mine) {
    const other = mine === "a" ? "b" : "a";
    player = (
      <p className={styles.note}>{t("waiting", { name: names[other] })}</p>
    );
  } else if (mine && request) {
    player = (
      <>
        <p>{t("asked", { name: names[request] })}</p>
        <Button
          size="sm"
          pending={action.pending}
          onClick={() => action.answer(true)}
        >
          {t("accept")}
        </Button>
        <Button
          size="sm"
          kind="secondary"
          pending={action.pending}
          onClick={() => action.answer(false)}
        >
          {t("decline")}
        </Button>
      </>
    );
  } else if (mine) {
    player = (
      <Button
        size="sm"
        kind="ghost"
        pending={action.pending}
        onClick={action.request}
      >
        {t("request")}
      </Button>
    );
  }

  return (
    <div className={styles.turn}>
      <div className={styles.buttons}>
        {player}
        {canManage && (
          <ConfirmButton
            question={t("organizerQuestion")}
            onConfirm={action.undo}
            pending={action.pending}
            size="sm"
          >
            {t("organizer")}
          </ConfirmButton>
        )}
      </div>
      {action.error && <ErrorText>{action.error}</ErrorText>}
    </div>
  );
}
