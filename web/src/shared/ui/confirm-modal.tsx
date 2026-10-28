"use client";

import { Modal, Stack, TextInput, usePrefix } from "@carbon/react";
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
  /** Um campo a mais depois da pergunta (a nota do histórico); recebe o foco inicial. */
  extra?: ReactNode;
  /** Mantém o primário travado (ex.: falta preencher o `extra`). */
  confirmDisabled?: boolean;
};

/**
 * A pergunta antes de uma ação que não se desfaz fácil: `Modal` do Carbon com
 * o título, o que vai acontecer, o primário com o nome da ação e "Não". O foco
 * começa no "Não", ou no primeiro campo quando há um (`extra` ou `slug`), para
 * um Enter a mais não confirmar. Quem abre cuida de devolver o foco ao fechar
 * (`ConfirmButton` e `OverflowActions` já fazem). Só existe enquanto aberto,
 * fora da árvore (no `body`), para não ficar um Modal escondido por linha de
 * tabela.
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
  extra,
  confirmDisabled,
}: ConfirmModalProps) {
  const t = useTranslations("Common");
  const prefix = usePrefix();
  const inputId = useId();
  const [typed, setTyped] = useState("");
  const blocked =
    confirmDisabled === true || (slug !== undefined && typed.trim() !== slug);
  const content = `.${prefix}--modal-content`;
  const hasField = slug !== undefined || extra !== undefined;

  const slugField = slug !== undefined && (
    <TextInput
      id={inputId}
      labelText={t("typeToConfirm", { slug })}
      autoComplete="off"
      spellCheck={false}
      value={typed}
      onChange={(event) => setTyped(event.target.value)}
    />
  );

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
        hasField
          ? `${content} input, ${content} select, ${content} textarea`
          : `.${prefix}--btn--secondary`
      }
      onRequestClose={onClose}
      onRequestSubmit={() => {
        if (!blocked) onConfirm();
      }}
    >
      {extra === undefined ? (
        <>
          <p>{children}</p>
          {slugField}
        </>
      ) : (
        <Stack gap={5}>
          <p>{children}</p>
          {extra}
          {slugField}
        </Stack>
      )}
    </Modal>
  );
}
