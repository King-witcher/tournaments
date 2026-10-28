"use client";

import { useTranslations } from "next-intl";
import { useRef, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { ConfirmModal } from "@/shared/ui/confirm-modal";
import { ErrorText } from "@/shared/ui/error-text";
import { TextInput } from "@/shared/ui/text-input";
import { gameParams, gameRowId, type MatchTarget } from "../lib/match-target";
import styles from "./game-replay.module.scss";

type Props = { target: MatchTarget; number: number };

/**
 * Para a organização, no último mapa ainda sem confirmar: anular para os
 * jogadores jogarem de novo (queda, revanche combinada). O botão abre o
 * `ConfirmModal` com a explicação e a nota do histórico; só o primário, com o mesmo
 * nome do botão, anula. Ao fechar, o foco volta ao botão; depois de anular,
 * à linha do mapa (que vira a do próximo envio, com o mesmo número).
 */
export function GameReplay({ target, number }: Props) {
  const t = useTranslations("Match.gameOrganizer");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const [open, setOpen] = useState(false);
  const [comment, setComment] = useState("");
  const trigger = useRef<HTMLButtonElement>(null);

  const close = () => {
    setOpen(false);
    // O Modal sai da árvore e o foco cairia no body: volta para o botão.
    requestAnimationFrame(() => trigger.current?.focus());
  };
  const replay = () => {
    close();
    run(
      () =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/replay",
          {
            params: gameParams(target, number),
            body: { comment: comment.trim() || null },
          },
        ),
      () => {
        setComment("");
        router.refresh();
        focusIfLost(gameRowId(number));
      },
      // Tela para trás (o mapa já confirmou pelo prazo): mostra o estado de agora.
      () => router.refresh(),
    );
  };

  return (
    <div className={styles.replay}>
      <Button
        ref={trigger}
        size="sm"
        kind="danger--tertiary"
        pending={pending}
        onClick={() => setOpen(true)}
      >
        {t("replay")}
      </Button>
      {error && <ErrorText>{error}</ErrorText>}
      <ConfirmModal
        open={open}
        onClose={close}
        onConfirm={replay}
        title={t("replaySubmit", { number })}
        confirmLabel={t("replay")}
        destructive
        extra={
          <TextInput
            labelText={t("note")}
            value={comment}
            onChange={(e) => setComment(e.target.value)}
            maxLength={280}
          />
        }
      >
        {t("replayQuestion", { number })}
      </ConfirmModal>
    </div>
  );
}
