import type { ReactNode } from "react";

import { Notice } from "./notice";

/**
 * O erro geral de um formulário (o que a API devolveu, já traduzido), acima
 * dos botões. O erro de um campo vai no `invalidText` do próprio campo.
 */
export function FormError({
  children,
  title,
  action,
}: {
  children: ReactNode;
  /** Resumo curto em negrito antes do texto. */
  title?: string;
  /** O que a pessoa pode fazer a respeito, embaixo do aviso (um `Button kind="ghost"`). */
  action?: ReactNode;
}) {
  return (
    <Notice kind="error" title={title} action={action}>
      {children}
    </Notice>
  );
}
