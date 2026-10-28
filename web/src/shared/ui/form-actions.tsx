"use client";

import { ButtonSet } from "@carbon/react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { Href } from "@/i18n/navigation";
import { Button } from "./button";
import { ButtonLink } from "./button-link";

type Props = {
  /** Texto do primário (`type="submit"`); padrão "Salvar". */
  submitLabel?: ReactNode;
  pending?: boolean;
  disabled?: boolean;
  /** Para onde "Cancelar" leva. */
  cancelHref?: Href;
  /** Ou o que "Cancelar" faz (fechar a edição). Sem os dois, não há "Cancelar". */
  onCancel?: () => void;
  cancelLabel?: ReactNode;
  /** Primário destrutivo (`danger`), ex.: "Remover senha". */
  destructive?: boolean;
};

/**
 * Os botões do fim de um formulário (`ButtonSet`): "Cancelar" secundário à
 * esquerda e o primário à direita, que envia o formulário e fica pendente.
 */
export function FormActions({
  submitLabel,
  pending,
  disabled,
  cancelHref,
  onCancel,
  cancelLabel,
  destructive,
}: Props) {
  const t = useTranslations("Common");
  const cancel = cancelLabel ?? t("cancel");
  return (
    <ButtonSet>
      {cancelHref !== undefined ? (
        <ButtonLink href={cancelHref} kind="secondary">
          {cancel}
        </ButtonLink>
      ) : (
        onCancel && (
          <Button kind="secondary" onClick={onCancel}>
            {cancel}
          </Button>
        )
      )}
      <Button
        type="submit"
        kind={destructive ? "danger" : "primary"}
        pending={pending}
        disabled={disabled}
      >
        {submitLabel ?? t("save")}
      </Button>
    </ButtonSet>
  );
}
