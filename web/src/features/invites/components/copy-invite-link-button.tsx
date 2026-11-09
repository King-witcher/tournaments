"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { useCopyToClipboard } from "@/shared/hooks/use-copy-to-clipboard";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button, type ButtonKind, type ButtonSize } from "@/shared/ui/button";
import { Toast } from "@/shared/ui/toast";

type Props = {
  /** O token de agora; volta do servidor quando alguém troca o link. */
  token: string;
  /** O nome do botão. */
  label: string;
  /** O que o toast diz depois de copiar. */
  copied: string;
  /** `info` quando o link copiado não vale agora (as vagas acabaram). */
  copiedKind?: "success" | "info";
  kind?: ButtonKind;
  size?: ButtonSize;
};

/**
 * Copia o link de convite ou de inscrição direto, ali onde a organização
 * está: `Button` + `Toast`, como o `CopyButton` indica para botão com texto.
 * Ver o link e trocá-lo ficam nas Configurações. O link vai sem idioma: o
 * proxy do next-intl manda cada pessoa para o seu.
 */
export function CopyInviteLinkButton({
  token,
  label,
  copied,
  copiedKind,
  kind,
  size,
}: Props) {
  const t = useTranslations("Invite");
  const { copy } = useCopyToClipboard();
  const [done, setDone] = useState(false);
  const [failed, setFailed] = useState(false);

  const onClick = () => {
    setFailed(false);
    // Sem clipboard (página sem HTTPS), `copy` lança: cai no mesmo aviso.
    Promise.resolve()
      .then(() => copy(`${window.location.origin}/invite/${token}`))
      .then(
        () => setDone(true),
        () => setFailed(true),
      );
  };

  return (
    <ActionWithNote error={failed ? t("copyFailed") : undefined}>
      <Button kind={kind} size={size} onClick={onClick}>
        {label}
      </Button>
      <Toast
        open={done}
        onClose={() => setDone(false)}
        title={copied}
        kind={copiedKind}
      />
    </ActionWithNote>
  );
}
