"use client";

import { type ReactNode, useCallback, useRef, useState } from "react";

import { Button, type ButtonKind, type ButtonSize } from "@/shared/ui/button";
import { CreateTenantModal } from "./create-tenant-modal";

type Props = {
  /** O texto do botão, também o título do Modal. */
  children: ReactNode;
  /** `primary` no estado vazio do início; `ghost` depois da lista. */
  kind?: ButtonKind;
  size?: ButtonSize;
};

/** "Criar comunidade" no início: abre a criação no Modal (`CreateTenantModal`). */
export function CreateTenantButton({
  children,
  kind = "primary",
  size,
}: Props) {
  const trigger = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(false);
  const close = useCallback(() => setOpen(false), []);

  return (
    <>
      <Button
        ref={trigger}
        kind={kind}
        size={size}
        onClick={() => setOpen(true)}
      >
        {children}
      </Button>
      {open && (
        <CreateTenantModal title={children} trigger={trigger} onClose={close} />
      )}
    </>
  );
}
