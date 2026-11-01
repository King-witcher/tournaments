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
  /** Uma ação a mais, na ponta oposta aos botões (ex.: um botão `ghost` ou `tertiary`). */
  extra?: ReactNode;
  /**
   * Outra forma de enviar, depois do primário ("Criar e publicar" depois de
   * "Criar como rascunho"): um `Button kind="secondary"`. Com ela, "Cancelar"
   * fica `ghost` e vai para o começo, para os três botões terem pesos
   * diferentes e a ordem ir do que não faz nada ao que faz mais.
   */
  alternative?: ReactNode;
};

/**
 * Os botões do fim de um formulário de página: o primário à esquerda, que
 * envia o formulário e fica pendente, e "Cancelar" secundário à direita
 * dele, como o Carbon pede fora de Modal e de assistente (o olho já está à
 * esquerda, lendo o formulário). Cada botão tem a largura do próprio texto.
 * `extra` é uma ação a mais na ponta oposta ("Salvar" e "Cancelar" à
 * esquerda, "Sortear" à direita). Com `alternative`, a ordem é "Cancelar",
 * o primário e a alternativa.
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
  const cancelButton =
    cancelHref !== undefined ? (
      <ButtonLink href={cancelHref} kind={cancelKind}>
        {cancel}
      </ButtonLink>
    ) : (
      onCancel && (
        <Button kind={cancelKind} onClick={onCancel}>
          {cancel}
        </Button>
      )
    );
  return (
    <div className={styles.actions}>
      <div className={styles.buttons}>
        {alternative && cancelButton}
        <Button
          ref={submitRef}
          type="submit"
          kind={destructive ? "danger" : "primary"}
          pending={pending}
          disabled={disabled}
        >
          {submitLabel ?? t("save")}
        </Button>
        {alternative}
        {!alternative && cancelButton}
      </div>
      {extra}
    </div>
  );
}
