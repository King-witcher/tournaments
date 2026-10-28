import { useState, useTransition } from "react";

import { type ApiResponse, callApi } from "./call";
import { useApiErrorMessage } from "./use-api-error-message";

/**
 * Ação de botão que chama a API. `run(chamada, aoDarCerto, aoFalhar?)` roda
 * numa transition; `pending` fica true enquanto isso e `error` recebe a
 * mensagem já traduzida (ou volta a `undefined` quando dá certo).
 * `aoFalhar` recebe o código, para quem precisa reagir ao erro (recarregar
 * os dados de uma tela que ficou para trás).
 */
export function useApiAction() {
  const errorMessage = useApiErrorMessage();
  const [pending, startTransition] = useTransition();
  const [error, setError] = useState<string>();

  function run<T>(
    request: () => Promise<ApiResponse<T>>,
    onSuccess: (data: T) => void,
    onError?: (code: string) => void,
  ) {
    startTransition(async () => {
      const outcome = await callApi(request());
      if (outcome.ok) {
        setError(undefined);
        onSuccess(outcome.data);
      } else {
        setError(errorMessage(outcome.code, outcome.params));
        onError?.(outcome.code);
      }
    });
  }

  return { run, pending, error };
}
