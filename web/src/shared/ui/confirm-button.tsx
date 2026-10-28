"use client";

import type { CarbonIconType } from "@carbon/icons-react";
import { type ReactNode, useRef, useState } from "react";

import { Button, type ButtonKind, type ButtonSize } from "./button";
import { ConfirmModal } from "./confirm-modal";

type Props = {
  /** Texto do botão; o mesmo vai no primário do Modal, para dizer o que acontece. */
  children: ReactNode;
  /** O que vai acontecer, numa frase, no corpo do Modal. */
  question: ReactNode;
  /** Título do Modal; sem ele, o texto do botão. */
  title?: ReactNode;
  onConfirm: () => void;
  pending?: boolean;
  disabled?: boolean;
  /** Apaga ou tira algo (padrão): Modal `danger`. `false` para o que só avança (publicar, iniciar). */
  destructive?: boolean;
  /** Grave e raro (desfazer início, transferir posse): pede este texto digitado. */
  confirmSlug?: string;
  /** Aparência do gatilho; padrão `danger--tertiary` (destrutivo) ou `tertiary`. */
  kind?: ButtonKind;
  size?: ButtonSize;
  renderIcon?: CarbonIconType;
  className?: string;
};

/**
 * Botão de ação que não se desfaz fácil: o clique abre o `ConfirmModal` (o que
 * vai acontecer, o primário com o mesmo texto do botão e "Não"); só o
 * primário roda `onConfirm`. Ao fechar, o foco volta para este botão, que
 * fica pendente enquanto a ação roda. Nada de `window.confirm`.
 */
export function ConfirmButton({
  children,
  question,
  title,
  onConfirm,
  pending,
  disabled,
  destructive = true,
  confirmSlug,
  kind,
  size = "sm",
  renderIcon,
  className,
}: Props) {
  const [open, setOpen] = useState(false);
  const trigger = useRef<HTMLButtonElement>(null);

  const close = () => {
    setOpen(false);
    // O Modal sai da árvore e o foco cairia no body: volta para o gatilho.
    requestAnimationFrame(() => trigger.current?.focus());
  };

  return (
    <>
      <Button
        ref={trigger}
        kind={kind ?? (destructive ? "danger--tertiary" : "tertiary")}
        size={size}
        pending={pending}
        disabled={disabled}
        renderIcon={renderIcon}
        className={className}
        onClick={() => setOpen(true)}
      >
        {children}
      </Button>
      <ConfirmModal
        open={open}
        onClose={close}
        onConfirm={() => {
          close();
          onConfirm();
        }}
        title={title ?? children}
        confirmLabel={children}
        destructive={destructive}
        slug={confirmSlug}
      >
        {question}
      </ConfirmModal>
    </>
  );
}
