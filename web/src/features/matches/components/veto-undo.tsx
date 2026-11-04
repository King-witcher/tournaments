"use client";

import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { Button } from "@/shared/ui/button";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { useVetoBoard } from "../hooks/use-veto-board";
import { otherSide } from "../lib/veto";
import styles from "./veto-undo.module.scss";

/**
 * Desfazer o último passo do jogo em andamento: quem pode fazer aquele tipo de
 * passo pede e, do outro lado, responde (um passo novo cancela o pedido); a
 * organização desfaz direto (inclusive o passo sorteado pelo fim do tempo).
 * Fica ao lado do título do passo: botões fantasma, para não competir com a
 * escolha.
 */
export function VetoUndo() {
  const t = useTranslations("Match.veto.undo");
  const {
    veto,
    mine,
    names,
    canManage,
    hasUndo,
    pending,
    actions: action,
  } = useVetoBoard();
  // Com um passo no ar, os botões ficam (sem ação) até a resposta dizer se
  // ainda há o que desfazer: sem pular de lugar.
  const shown = veto?.can_undo || pending === "step";
  if (!hasUndo || !veto || !shown || (!mine && !canManage)) return null;
  const busy = (key: string) =>
    pending === key ? { pending: true } : { disabled: pending != null };
  const request = veto.undo_request;
  const can = veto.can_act.undo || pending === "step";

  let player: ReactNode = null;
  if (mine && can && request === mine) {
    player = (
      <p className={styles.note}>
        {t("waiting", { name: names[otherSide(mine)] })}
      </p>
    );
  } else if (mine && can && request) {
    player = (
      <>
        <p>{t("asked", { name: names[request] })}</p>
        <Button
          size="sm"
          {...busy("accept")}
          onClick={() => action.answer(true)}
        >
          {t("accept")}
        </Button>
        <Button
          size="sm"
          kind="secondary"
          {...busy("decline")}
          onClick={() => action.answer(false)}
        >
          {t("decline")}
        </Button>
      </>
    );
  } else if (mine && can) {
    player = (
      <Button
        size="sm"
        kind="ghost"
        {...busy("request")}
        onClick={action.request}
      >
        {t("request")}
      </Button>
    );
  }

  if (!player && !canManage) return null;

  return (
    <div className={styles.undo}>
      {player}
      {canManage && (
        <ConfirmButton
          question={t("organizerQuestion")}
          onConfirm={action.undo}
          {...busy("undo")}
          kind="danger--ghost"
          size="sm"
        >
          {t("organizer")}
        </ConfirmButton>
      )}
    </div>
  );
}
