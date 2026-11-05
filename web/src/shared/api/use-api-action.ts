import { startTransition, useRef, useState, useTransition } from "react";

import { type ApiOutcome, type ApiResponse, callApi } from "./call";
import { useApiErrorMessage } from "./use-api-error-message";

/**
 * Ação de botão que chama a API. `run(chamada, aoDarCerto, aoFalhar?)` roda
 * numa transition, e `pending` fica true do clique até a tela mostrar o que
 * `aoDarCerto` mudou: ele roda dentro da mesma transition, então o
 * `router.refresh()` dele e o estado que ele troca (fechar o modal, limpar um
 * campo) aparecem juntos, com a página nova, sem um intervalo em que o botão
 * já parou e a tela ainda é a velha. `error` recebe a mensagem já traduzida
 * (ou volta a `undefined` quando dá certo). `aoFalhar` recebe o código, para
 * quem precisa reagir ao erro (recarregar os dados de uma tela que ficou para
 * trás). Com um pedido no ar, outro `run` do mesmo hook é ignorado (o clique
 * duplo antes de o botão ficar pendente).
 */
export function useApiAction() {
  const errorMessage = useApiErrorMessage();
  const [pending, startAction] = useTransition();
  const [error, setError] = useState<string>();
  const busy = useRef(false);

  function run<T>(
    request: () => Promise<ApiResponse<T>>,
    onSuccess: (data: T) => void,
    onError?: (code: string) => void,
  ) {
    if (busy.current) return;
    busy.current = true;
    startAction(async () => {
      let outcome: ApiOutcome<T>;
      try {
        outcome = await callApi(request());
      } finally {
        busy.current = false;
      }
      if (outcome.ok) {
        const { data } = outcome;
        setError(undefined);
        // Depois do `await`, só entra na transition (e no `pending`) o que
        // roda dentro de outra.
        startTransition(() => onSuccess(data));
      } else {
        setError(errorMessage(outcome.code, outcome.params));
        onError?.(outcome.code);
      }
    });
  }

  return { run, pending, error };
}
