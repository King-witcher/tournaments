"use client";

import { useTranslations } from "next-intl";
import type { ReactNode, Ref } from "react";

import type { Href } from "@/i18n/navigation";
import { Button } from "./button";
import { ButtonLink } from "./button-link";
import styles from "./form-actions.module.scss";

type Props = {
  /** Texto do primário (`type="submit"`); padrão "Salvar". */
  submitLabel?: ReactNode;
  /** O primário, para devolver o foco a ele (depois de um Modal que ele abriu). */
  submitRef?: Ref<HTMLButtonElement>;
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
  /**
   * Outra forma de enviar, entre "Cancelar" e o primário ("Criar como
   * rascunho" ao lado de "Criar"): um `Button kind="secondary"`. Com ela,
   * "Cancelar" fica `ghost`, para os três botões terem pesos diferentes.
   */
  alternative?: ReactNode;
};

/**
 * Os botões do fim de um formulário: "Cancelar" secundário à esquerda e o
 * primário à direita, que envia o formulário e fica pendente. Cada botão tem
 * a largura do próprio texto. `extra` é uma ação a mais na ponta oposta
 * ("Sortear" ao lado de "Cancelar" e "Salvar").
 */
export function FormActions({
  submitLabel,
  submitRef,
  pending,
  disabled,
  cancelHref,
  onCancel,
  cancelLabel,
  destructive,
  extra,
  alternative,
}: Props) {
  const t = useTranslations("Common");
  const cancel = cancelLabel ?? t("cancel");
  const cancelKind = alternative ? "ghost" : "secondary";
  return (
    <div className={styles.actions}>
      {extra}
      <div className={styles.buttons}>
        {cancelHref !== undefined ? (
          <ButtonLink href={cancelHref} kind={cancelKind}>
            {cancel}
          </ButtonLink>
        ) : (
          onCancel && (
            <Button kind={cancelKind} onClick={onCancel}>
              {cancel}
            </Button>
          )
        )}
        {alternative}
        <Button
          ref={submitRef}
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
