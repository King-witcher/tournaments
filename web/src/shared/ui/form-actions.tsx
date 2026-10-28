"use client";

import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { Href } from "@/i18n/navigation";
import { Button } from "./button";
import { ButtonLink } from "./button-link";
import styles from "./form-actions.module.scss";

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
  /** Uma ação a mais, à esquerda de "Cancelar" e do primário (ex.: um botão `ghost` ou `tertiary`). */
  extra?: ReactNode;
};

/**
 * Os botões do fim de um formulário: "Cancelar" secundário à esquerda e o
 * primário à direita, que envia o formulário e fica pendente. Cada botão tem
 * a largura do próprio texto. `extra` é uma ação a mais na ponta oposta
 * ("Sortear" ao lado de "Cancelar" e "Salvar").
 */
export function FormActions({
  submitLabel,
  pending,
  disabled,
  cancelHref,
  onCancel,
  cancelLabel,
  destructive,
  extra,
}: Props) {
  const t = useTranslations("Common");
  const cancel = cancelLabel ?? t("cancel");
  return (
    <div className={styles.actions}>
      {extra}
      <div className={styles.buttons}>
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
      </div>
    </div>
  );
}
