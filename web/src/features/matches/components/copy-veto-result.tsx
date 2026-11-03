"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { useCopyToClipboard } from "@/shared/hooks/use-copy-to-clipboard";
import { Button, type ButtonKind, type ButtonSize } from "@/shared/ui/button";
import { Toast } from "@/shared/ui/toast";
import { useVetoResult } from "../hooks/use-veto-result";

type Props = {
  kind?: ButtonKind;
  size?: ButtonSize;
};

/**
 * Copia o resultado do veto para colar no chat, com o aviso de que copiou.
 * Discreto na série, onde o que importa é a partida da vez; primário no passo
 * de resultado da sala de veto. Some sem nada para copiar.
 */
export function CopyVetoResult({ kind = "ghost", size = "sm" }: Props) {
  const t = useTranslations("Match.veto.result");
  const result = useVetoResult();
  const { copy } = useCopyToClipboard();
  const [copied, setCopied] = useState(false);
  if (!result) return null;
  return (
    <div>
      <Button
        kind={kind}
        size={size}
        onClick={() => {
          copy(result).then(() => setCopied(true));
        }}
      >
        {t("copy")}
      </Button>
      <Toast
        open={copied}
        onClose={() => setCopied(false)}
        title={t("copied")}
      />
    </div>
  );
}
