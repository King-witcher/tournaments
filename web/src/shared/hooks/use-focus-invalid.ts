import { type RefObject, useEffect } from "react";

import { type FormResult, fieldId } from "@/shared/lib/form-result";

/**
 * Depois de um envio recusado por causa de um campo, leva o foco a ele: o
 * leitor de tela lê o rótulo e o erro, e quem digita corrige sem procurar.
 * `scope` diz onde achar o campo: o `prefix` do `useId` (os campos têm o id
 * `fieldId(prefix, campo)`) ou a ref do formulário (os campos têm o `name`
 * igual ao campo). Roda a cada resultado novo do envio: o `result` precisa
 * mudar de identidade só quando há um envio novo (o do `useActionState`).
 */
export function useFocusInvalid<F extends string>(
  scope: string | RefObject<HTMLElement | null>,
  result: FormResult<F>,
) {
  useEffect(() => {
    if (!result.field) return;
    const control =
      typeof scope === "string"
        ? document.getElementById(fieldId(scope, result.field))
        : scope.current?.querySelector<HTMLElement>(`[name="${result.field}"]`);
    control?.focus();
  }, [scope, result]);
}
