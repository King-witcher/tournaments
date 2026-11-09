"use client";

import { useTranslations } from "next-intl";
import { useRef, useState } from "react";

import { Button, type ButtonKind } from "@/shared/ui/button";
import { JoinWithCodeModal } from "./join-with-code-modal";

/**
 * "Entrar com código": no aplicativo instalado não há barra de endereço e os
 * links abrem no navegador, então quem recebe um convite o digita aqui.
 */
export function JoinWithCodeButton({
  kind = "tertiary",
}: {
  kind?: ButtonKind;
}) {
  const t = useTranslations("Invite.join");
  const trigger = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(false);

  return (
    <>
      <Button ref={trigger} kind={kind} onClick={() => setOpen(true)}>
        {t("open")}
      </Button>
      {open && (
        <JoinWithCodeModal trigger={trigger} onClose={() => setOpen(false)} />
      )}
    </>
  );
}
