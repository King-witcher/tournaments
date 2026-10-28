"use client";

import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useId, useRef, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { ErrorText } from "@/shared/ui/error-text";
import { TextArea } from "@/shared/ui/text-area";
import { gameParams, gameRowId, type MatchTarget } from "../lib/match-target";
import styles from "./game-respond.module.scss";

type Props = { target: MatchTarget; number: number };

/** O comentário da contestação vai de 1 a 280 caracteres. */
const MAX_COMMENT = 280;

/** Para o adversário de quem enviou o mapa: confirmar, ou contestar com um comentário. */
export function GameRespond({ target, number }: Props) {
  const t = useTranslations("Match.respond");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const [contesting, setContesting] = useState(false);
  const [comment, setComment] = useState("");
  const fieldId = useId();
  const contestButton = useRef<HTMLButtonElement>(null);
  const params = gameParams(target, number);
  const done = () => {
    router.refresh();
    focusIfLost(gameRowId(number));
  };
  // A tela pode ter ficado para trás (o mapa já foi confirmado pela
  // organização, ou anulado): recarrega para mostrar o estado de agora.
  const stale = () => router.refresh();

  const confirm = () =>
    run(
      () =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/confirm",
          { params },
        ),
      done,
      stale,
    );
  const contest = () =>
    run(
      () =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/contest",
          { params, body: { comment } },
        ),
      done,
      stale,
    );
  // O botão que abriu some: o foco vai para o campo do comentário e, ao
  // cancelar, volta para o "Contestar".
  const open = () => {
    setContesting(true);
    requestAnimationFrame(() => document.getElementById(fieldId)?.focus());
  };
  const cancel = () => {
    setContesting(false);
    requestAnimationFrame(() => contestButton.current?.focus());
  };

  return (
    <div className={styles.respond}>
      {contesting ? (
        <Stack gap={5}>
          <TextArea
            id={fieldId}
            labelText={t("comment")}
            helperText={t("commentHint")}
            value={comment}
            onChange={(e) => setComment(e.target.value)}
            rows={2}
            enableCounter
            maxCount={MAX_COMMENT}
          />
          <div className={styles.buttons}>
            <Button size="sm" pending={pending} onClick={contest}>
              {t("submitContest")}
            </Button>
            <Button size="sm" kind="secondary" onClick={cancel}>
              {t("cancel")}
            </Button>
          </div>
        </Stack>
      ) : (
        <div className={styles.buttons}>
          <Button size="sm" pending={pending} onClick={confirm}>
            {t("confirm")}
          </Button>
          <Button ref={contestButton} size="sm" kind="tertiary" onClick={open}>
            {t("contest")}
          </Button>
        </div>
      )}
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
