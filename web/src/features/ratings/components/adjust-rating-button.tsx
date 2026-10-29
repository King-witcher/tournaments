"use client";

import { useTranslations } from "next-intl";
import { type RefObject, useRef, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { FormModal } from "@/shared/ui/form-modal";
import { NumberInput } from "@/shared/ui/number-input";
import { TextInput } from "@/shared/ui/text-input";

/** Os mesmos limites da API. */
const MAX_DELTA = 1000;
const MAX_REASON = 280;

type Props = {
  tenant: string;
  userId: number;
  nickname: string;
  /** O bloco que recebe o foco depois do ajuste. */
  focusId: string;
};

/**
 * "Ajustar rating", para a organização, na página da pessoa: abre o
 * `FormModal` com os pontos e o motivo. Ajustado, a página recarrega e o
 * foco vai para o resumo do rating.
 */
export function AdjustRatingButton({
  tenant,
  userId,
  nickname,
  focusId,
}: Props) {
  const t = useTranslations("Ratings.adjustments");
  const router = useRouter();
  const trigger = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(false);

  return (
    <>
      <Button
        ref={trigger}
        kind="tertiary"
        size="sm"
        onClick={() => setOpen(true)}
      >
        {t("open")}
      </Button>
      {open && (
        <AdjustRatingModal
          tenant={tenant}
          userId={userId}
          nickname={nickname}
          trigger={trigger}
          onClose={() => setOpen(false)}
          onDone={() => {
            setOpen(false);
            router.refresh();
            focusIfLost(focusId);
          }}
        />
      )}
    </>
  );
}

/** O Modal aberto: um a cada abertura, para começar vazio. */
function AdjustRatingModal({
  tenant,
  userId,
  nickname,
  trigger,
  onClose,
  onDone,
}: Omit<Props, "focusId"> & {
  trigger: RefObject<HTMLButtonElement | null>;
  onClose: () => void;
  onDone: () => void;
}) {
  const t = useTranslations("Ratings.adjustments");
  const { run, pending, error } = useApiAction();
  const [delta, setDelta] = useState<number | null>(null);
  const [reason, setReason] = useState("");
  const valid =
    delta !== null &&
    Number.isInteger(delta) &&
    delta !== 0 &&
    Math.abs(delta) <= MAX_DELTA &&
    reason.trim() !== "";

  return (
    <FormModal
      title={t("modalTitle", { name: nickname })}
      submitLabel={t("submit")}
      pending={pending}
      submitDisabled={!valid}
      error={error}
      returnFocusTo={trigger}
      onClose={onClose}
      onSubmit={() =>
        run(
          () =>
            api.POST("/tenants/{slug}/ratings/{user_id}/adjustments", {
              params: { path: { slug: tenant, user_id: userId } },
              body: { delta: delta ?? 0, reason },
            }),
          onDone,
        )
      }
    >
      <p>{t("hint")}</p>
      <NumberInput
        labelText={t("delta")}
        helperText={t("deltaHint")}
        allowEmpty
        hideSteppers
        disableWheel
        min={-MAX_DELTA}
        max={MAX_DELTA}
        step={1}
        value={delta}
        onChange={setDelta}
      />
      <TextInput
        labelText={t("reasonLabel")}
        helperText={t("reasonHint")}
        maxLength={MAX_REASON}
        value={reason}
        onChange={(event) => setReason(event.target.value)}
      />
    </FormModal>
  );
}
