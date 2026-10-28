"use client";

import { Modal, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useRef, useState } from "react";
import { createPortal } from "react-dom";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { TextInput } from "@/shared/ui/text-input";
import { gameParams, gameRowId, type MatchTarget } from "../lib/match-target";
import styles from "./game-replay.module.scss";

type Props = { target: MatchTarget; number: number };

/**
 * Para a organização, no último mapa ainda sem confirmar: anular para os
 * jogadores jogarem de novo (queda, revanche combinada). O botão abre o
 * `Modal` com a explicação e a nota do histórico; só o primário, com o mesmo
 * nome do botão, anula. Ao fechar, o foco volta ao botão; depois de anular,
 * à linha do mapa (que vira a do próximo envio, com o mesmo número).
 */
export function GameReplay({ target, number }: Props) {
  const t = useTranslations("Match.gameOrganizer");
  const tCommon = useTranslations("Common");
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
      {open &&
        createPortal(
          <Modal
            open
            size="sm"
            danger
            modalHeading={t("replaySubmit", { number })}
            primaryButtonText={t("replay")}
            secondaryButtonText={tCommon("no")}
            closeButtonLabel={tCommon("close")}
            selectorPrimaryFocus="[data-modal-primary-focus]"
            onRequestClose={close}
            onRequestSubmit={replay}
          >
            <Stack gap={5}>
              <p>{t("replayQuestion", { number })}</p>
              <TextInput
                data-modal-primary-focus
                labelText={t("note")}
                value={comment}
                onChange={(e) => setComment(e.target.value)}
                maxLength={280}
              />
            </Stack>
          </Modal>,
          document.body,
        )}
    </div>
  );
}
