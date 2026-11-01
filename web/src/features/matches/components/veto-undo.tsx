"use client";

import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { Button } from "@/shared/ui/button";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { ErrorText } from "@/shared/ui/error-text";
import { useLiveMatch } from "../hooks/use-live-match";
import { useSideNames } from "../hooks/use-side-names";
import { useVetoAction } from "../hooks/use-veto-action";
import type { MatchTarget } from "../lib/match-target";
import { otherSide } from "../lib/veto";
import styles from "./veto-turn.module.scss";

/**
 * Desfazer o último passo do jogo em andamento: o jogador pede e o
 * adversário aceita ou recusa (um passo novo cancela o pedido); a
 * organização desfaz direto (inclusive o passo sorteado pelo fim do tempo).
 */
export function VetoUndo({ target }: { target: MatchTarget }) {
  const t = useTranslations("Match.veto.undo");
  const { detail } = useLiveMatch();
  const names = useSideNames(detail);
  const action = useVetoAction(target);
  const veto = detail.veto;
  const mine = detail.viewer_side ?? null;
  const canManage = detail.can_manage;
  if (!veto?.can_undo || (!mine && !canManage)) return null;
  const request = veto.undo_request;

  let player: ReactNode = null;
  if (mine && request === mine) {
    player = (
      <p className={styles.note}>
        {t("waiting", { name: names[otherSide(mine)] })}
      </p>
    );
  } else if (mine && request) {
    player = (
      <>
        <p>{t("asked", { name: names[request] })}</p>
        <Button
          size="sm"
          pending={action.pending === "accept"}
          onClick={() => action.answer(true)}
        >
          {t("accept")}
        </Button>
        <Button
          size="sm"
          kind="secondary"
          pending={action.pending === "decline"}
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
        pending={action.pending === "request"}
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
            pending={action.pending === "undo"}
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
