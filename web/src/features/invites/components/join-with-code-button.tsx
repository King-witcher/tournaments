"use client";

import { useTranslations } from "next-intl";
import { useRef, useState } from "react";

import { Button, type ButtonKind, type ButtonSize } from "@/shared/ui/button";
import { JoinWithCodeModal } from "./join-with-code-modal";

/**
 * "Entrar em comunidade", pelo convite: no aplicativo instalado não há barra
 * de endereço e os links abrem no navegador, então quem recebe um convite o
 * digita aqui.
 */
export function JoinWithCodeButton({
  kind = "tertiary",
  size,
}: {
  kind?: ButtonKind;
  size?: ButtonSize;
}) {
  const t = useTranslations("Invite.join");
  const trigger = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(false);

  return (
    <>
      <Button
        ref={trigger}
        kind={kind}
        size={size}
        onClick={() => setOpen(true)}
      >
        {t("open")}
      </Button>
      {open && (
        <JoinWithCodeModal trigger={trigger} onClose={() => setOpen(false)} />
      )}
    </>
  );
}
