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
 *
 * Vários botões no mesmo hook: `runAs(chave, chamada, aoDarCerto, aoFalhar?)`
 * e, em cada botão, `{...buttonFor(chave)}`. Só o botão da chave em andamento
 * mostra o indicador; os outros ficam desabilitados, sem ele, como o Carbon
 * desabilita as demais ações enquanto uma carrega.
 */
export function useApiAction() {
  const errorMessage = useApiErrorMessage();
  const [pending, startAction] = useTransition();
  const [error, setError] = useState<string>();
  const [runningKey, setRunningKey] = useState<string>();
  const busy = useRef(false);

  function runAs<T>(
    key: string | undefined,
    request: () => Promise<ApiResponse<T>>,
    onSuccess: (data: T) => void,
    onError?: (code: string) => void,
  ) {
    if (busy.current) return;
    busy.current = true;
    setRunningKey(key);
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

  function run<T>(
    request: () => Promise<ApiResponse<T>>,
    onSuccess: (data: T) => void,
    onError?: (code: string) => void,
  ) {
    runAs(undefined, request, onSuccess, onError);
  }

  /** As props de um botão de um grupo (`Button`, `ConfirmButton`). */
  function buttonFor(key: string) {
    return {
      pending: pending && runningKey === key,
      disabled: pending && runningKey !== key,
    };
  }

  return { run, runAs, buttonFor, pending, error };
}
