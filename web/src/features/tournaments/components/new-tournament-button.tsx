"use client";

import { type ReactNode, useCallback, useRef, useState } from "react";

import { Button, type ButtonKind } from "@/shared/ui/button";
import { NewTournamentModal } from "./new-tournament-modal";

type Props = {
  tenant: string;
  /** O texto do botão, também o título do Modal. */
  children: ReactNode;
  /** `primary` no cabeçalho da comunidade; `tertiary` no estado vazio da lista. */
  kind?: ButtonKind;
};

/** "Novo torneio": abre a criação no Modal (`NewTournamentModal`). */
export function NewTournamentButton({
  tenant,
  children,
  kind = "primary",
}: Props) {
  const trigger = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(false);
  const close = useCallback(() => setOpen(false), []);

  return (
    <>
      <Button ref={trigger} kind={kind} onClick={() => setOpen(true)}>
        {children}
      </Button>
      {open && (
        <NewTournamentModal
          tenant={tenant}
          title={children}
          trigger={trigger}
          onClose={close}
        />
      )}
    </>
  );
}
