"use client";

import { Modal, TextInput, usePrefix } from "@carbon/react";
import { useTranslations } from "next-intl";
import { type ReactNode, useId, useState } from "react";
import { createPortal } from "react-dom";

export type ConfirmModalProps = {
  open: boolean;
  /** Fechou sem confirmar (Não, Esc, X ou clique fora). */
  onClose: () => void;
  onConfirm: () => void;
  /** Título curto (o nome da ação). */
  title: ReactNode;
  /** O que vai acontecer, numa ou duas frases. */
  children: ReactNode;
  /** Texto do botão primário: o mesmo do botão que abriu. */
  confirmLabel: ReactNode;
  /** Apaga ou tira algo: Modal `danger` (primário vermelho). */
  destructive?: boolean;
  /** Para o que é grave e raro: o primário só libera com este texto digitado. */
  slug?: string;
};

/**
 * A pergunta antes de uma ação que não se desfaz fácil: `Modal` do Carbon com
 * o título, o que vai acontecer, o primário com o nome da ação e "Não". O foco
 * começa no "Não" (ou no campo do slug), para um Enter a mais não confirmar.
 * Quem abre cuida de devolver o foco ao fechar (`ConfirmButton` e
 * `OverflowActions` já fazem). Só existe enquanto aberto, fora da árvore (no
 * `body`), para não ficar um Modal escondido por linha de tabela.
 */
export function ConfirmModal(props: ConfirmModalProps) {
  if (!props.open) return null;
  return createPortal(<OpenConfirmModal {...props} />, document.body);
}

function OpenConfirmModal({
  onClose,
  onConfirm,
  title,
  children,
  confirmLabel,
  destructive,
  slug,
}: ConfirmModalProps) {
  const t = useTranslations("Common");
  const prefix = usePrefix();
  const inputId = useId();
  const [typed, setTyped] = useState("");
  const blocked = slug !== undefined && typed.trim() !== slug;

  return (
    <Modal
      open
      size="sm"
      danger={destructive}
      modalHeading={title}
      primaryButtonText={confirmLabel}
      secondaryButtonText={t("no")}
      closeButtonLabel={t("close")}
      primaryButtonDisabled={blocked}
      selectorPrimaryFocus={
        slug === undefined
          ? `.${prefix}--btn--secondary`
          : "[data-modal-primary-focus]"
      }
      onRequestClose={onClose}
      onRequestSubmit={() => {
        if (!blocked) onConfirm();
      }}
    >
      <p>{children}</p>
      {slug !== undefined && (
        <TextInput
          id={inputId}
          data-modal-primary-focus
          labelText={t("typeToConfirm", { slug })}
          autoComplete="off"
          spellCheck={false}
          value={typed}
          onChange={(event) => setTyped(event.target.value)}
        />
      )}
    </Modal>
  );
}
