import { useEffect } from "react";

import type { AuthField, FormResult } from "../lib/form-result";

/**
 * Depois de um envio recusado por causa de um campo, leva o foco a ele: o
 * leitor de tela lê o rótulo e o erro, e quem digita corrige sem procurar. Os
 * campos do formulário têm o id `${prefix}${campo}` (`prefix` vem do
 * `useId`). Roda a cada resultado novo do envio.
 */
export function useFocusInvalid(prefix: string, result: FormResult) {
  useEffect(() => {
    if (result.field)
      document.getElementById(`${prefix}${result.field}`)?.focus();
  }, [prefix, result]);
}

/** O id do campo `field` num formulário com o `prefix` do `useId`. */
export function fieldId(prefix: string, field: AuthField) {
  return `${prefix}${field}`;
}
