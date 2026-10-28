import type { ReactNode } from "react";

import { Notice } from "./notice";

/**
 * O erro geral de um formulário (o que a API devolveu, já traduzido), acima
 * dos botões. O erro de um campo vai no `invalidText` do próprio campo.
 */
export function FormError({
  children,
  title,
}: {
  children: ReactNode;
  /** Resumo curto em negrito antes do texto. */
  title?: string;
}) {
  return (
    <Notice kind="error" title={title}>
      {children}
    </Notice>
  );
}
