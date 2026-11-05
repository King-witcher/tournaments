"use client";

import { useTranslations } from "next-intl";
import type { ComponentProps, MouseEvent } from "react";

import { useCopyToClipboard } from "@/shared/hooks/use-copy-to-clipboard";
import { CopyButton as CarbonCopyButton } from "./carbon/copy-button";

type Props = ComponentProps<typeof CarbonCopyButton> & {
  /** O que vai para a área de transferência ao clicar. */
  text: string;
};

/**
 * Botão só com ícone que copia um texto (`CopyButton` do Carbon), ao lado do
 * que ele copia quando isso já está à vista (um código, um id). Confirma com
 * a dica "Copiado" no próprio botão; `align` escolhe o lado dela e
 * `feedbackTimeout`, quanto tempo fica. Texto que a pessoa precisa ver e
 * copiar inteiro (um link de convite): `CodeSnippet`; botão com texto
 * ("Copiar link") que avisa num toast: `Button` + `Toast`.
 *
 * Copia o `text` (o do Carbon só mostra a dica) e traduz o nome e a dica.
 */
export function CopyButton({
  text,
  onClick,
  feedback,
  iconDescription,
  ...props
}: Props) {
  const t = useTranslations("Common.copy");
  const { copy } = useCopyToClipboard();
  return (
    <CarbonCopyButton
      feedback={feedback ?? t("copied")}
      iconDescription={iconDescription ?? t("copy")}
      onClick={(event: MouseEvent<HTMLButtonElement>) => {
        copy(text);
        onClick?.(event);
      }}
      {...props}
    />
  );
}
