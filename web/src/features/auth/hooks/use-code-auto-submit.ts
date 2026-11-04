import { type RefObject, useEffect, useRef } from "react";

import { CODE_LENGTH } from "../components/code-input";

/**
 * Envia o formulário sozinho ao completar o código, uma vez por código: o mesmo
 * só vai de novo pelo botão. `enabled` falso espera o botão (há outros campos
 * para preencher).
 */
export function useCodeAutoSubmit(
  code: string,
  form: RefObject<HTMLFormElement | null>,
  enabled = true,
) {
  const sent = useRef<string>(undefined);
  useEffect(() => {
    if (code.length < CODE_LENGTH) sent.current = undefined;
    else if (enabled && sent.current !== code) {
      sent.current = code;
      form.current?.requestSubmit();
    }
  }, [code, form, enabled]);
}
