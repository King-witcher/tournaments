"use client";

import { useTranslations } from "next-intl";
import { type RefObject, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { FormModal } from "@/shared/ui/form-modal";
import { RadioGroup } from "@/shared/ui/radio-group";
import { TextInput } from "@/shared/ui/text-input";
import { type MatchTarget, matchParams, SCORE_ID } from "../lib/match-target";

type Side = "a" | "b";

type Props = {
  target: MatchTarget;
  names: { a: string; b: string };
  /** A série já tem resultado: o W.O. fica no lugar dele. */
  decided: boolean;
  /** O botão que abriu, para o foco voltar a ele. */
  trigger: RefObject<HTMLButtonElement | null>;
  onClose: () => void;
};

/**
 * O W.O. da organização: quem vence (sem nada marcado: a escolha precisa ser
 * de quem preenche), o que acontece com essa escolha e a nota opcional.
 * Depois, o foco vai para a linha do resultado.
 */
export function WalkoverModal({
  target,
  names,
  decided,
  trigger,
  onClose,
}: Props) {
  const t = useTranslations("Match.organizer");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const [winner, setWinner] = useState<Side>();
  const [comment, setComment] = useState("");

  const submit = () => {
    if (!winner) return;
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
        onClose();
        router.refresh();
        focusIfLost(SCORE_ID);
      },
    );
  };

  return (
    <FormModal
      title={t("walkoverTitle")}
      submitLabel={t("walkoverSubmit")}
      onSubmit={submit}
      onClose={onClose}
      pending={pending}
      submitDisabled={!winner}
      error={error}
      returnFocusTo={trigger}
    >
      <RadioGroup
        legendText={t("winner")}
        value={winner}
        onChange={setWinner}
        options={[
          { value: "a", label: names.a },
          { value: "b", label: names.b },
        ]}
      />
      {winner && (
        <p>
          {t(decided ? "walkoverEffectDecided" : "walkoverEffect", {
            name: names[winner],
          })}
        </p>
      )}
      <TextInput
        labelText={t("comment")}
        value={comment}
        onChange={(e) => setComment(e.target.value)}
        maxLength={280}
      />
    </FormModal>
  );
}
