"use client";

import { FormGroup, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { ErrorText } from "@/shared/ui/error-text";
import { TextInput } from "@/shared/ui/text-input";
import { type MatchTarget, matchParams, SCORE_ID } from "../lib/match-target";
import styles from "./walkover-form.module.scss";

type Props = { target: MatchTarget; names: { a: string; b: string } };

/**
 * Para a organização: W.O. para um dos lados, com nota opcional. A
 * confirmação é a de quem só avança (primário na cor de sempre). Depois, o
 * bloco fecha com o botão usado e o foco vai para o placar da série.
 */
export function WalkoverForm({ target, names }: Props) {
  const t = useTranslations("Match.organizer");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const [comment, setComment] = useState("");

  const give = (winner: "a" | "b") => {
    run(
      () =>
        api.PUT(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/result",
          {
            params: matchParams(target),
            body: {
              result: { kind: "walkover", winner },
              comment: comment.trim() || null,
            },
          },
        ),
      () => {
        router.refresh();
        focusIfLost(SCORE_ID);
      },
    );
  };

  return (
    <FormGroup legendText={t("walkoverTitle")}>
      <Stack gap={5}>
        <div className={styles.field}>
          <TextInput
            labelText={t("comment")}
            value={comment}
            onChange={(e) => setComment(e.target.value)}
            maxLength={280}
          />
        </div>
        <div className={styles.buttons}>
          {(["a", "b"] as const).map((side) => (
            <ConfirmButton
              key={side}
              destructive={false}
              pending={pending}
              question={t("walkoverConfirm", { name: names[side] })}
              onConfirm={() => give(side)}
            >
              {t("walkover", { name: names[side] })}
            </ConfirmButton>
          ))}
        </div>
        {error && <ErrorText>{error}</ErrorText>}
      </Stack>
    </FormGroup>
  );
}
