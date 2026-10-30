import { useEffect, useRef, useState } from "react";

/**
 * Diz se o formulário pode ser enviado: todos os campos válidos
 * (`form.checkValidity()`, que já conta o que os campos de `shared/ui`
 * declaram). Ponha `ref` no `<Form>` e `disabled={!canSubmit}` no
 * `FormActions`. Começa falso: até hidratar, o envio fica travado.
 */
export function useFormValidity() {
  const ref = useRef<HTMLFormElement>(null);
  const [canSubmit, setCanSubmit] = useState(false);
  // A cada render: os efeitos dos campos (filhos) rodam antes e já gravaram a
  // validade deles; um mesmo valor não renderiza de novo.
  useEffect(() => {
    setCanSubmit(ref.current?.checkValidity() ?? false);
  });
  return { ref, canSubmit };
}
