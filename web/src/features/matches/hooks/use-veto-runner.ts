"use client";

import { useRef, useState } from "react";

import { useRealtimeHub } from "@/features/realtime/hooks/use-realtime-hub";
import { useRouter } from "@/i18n/navigation";
import { type ApiResponse, callApi } from "@/shared/api/call";
import type { components } from "@/shared/api/schema";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { focusIfLost } from "@/shared/lib/focus";
import { noteServerTime } from "@/shared/lib/server-clock";
import { VETO_ID } from "../lib/match-target";

type Schemas = components["schemas"];

/** O que a ação recebe para montar o pedido. */
export type VetoRequestInit = {
  headers: Record<string, string>;
  /** A vez que a tela mostrava no clique. */
  step: number | null;
};

export type VetoRequest = (
  init: VetoRequestInit,
) => Promise<ApiResponse<Schemas["MatchDetail"]>>;

type Props = {
  /** A série do servidor (muda a cada recarga). */
  initial: Schemas["MatchDetail"];
  /** A série que a tela mostra, sem o passo marcado na hora. */
  shown: Schemas["MatchDetail"];
  adopt: (detail: Schemas["MatchDetail"]) => void;
  preview: (veto: Schemas["VetoView"] | null) => void;
};

/**
 * As ações do veto da série, uma por vez para todos os tabuleiros (uma
 * segunda, com outra no ar, sairia com a vez velha): \`pending\` é a chave da
 * que está no ar e \`error\`, a mensagem da que falhou, que fica até a vez
 * mudar depois da recarga que o erro dispara (o "a vez mudou antes do seu
 * clique" fala justamente da vez nova), mesmo que o tabuleiro que agiu
 * tenha saído da tela. A resposta mede a hora da API pela ida e volta.
 */
export function useVetoRunner({ initial, shown, adopt, preview }: Props) {
  const router = useRouter();
  const hub = useRealtimeHub();
  const errorMessage = useApiErrorMessage();
  const [pending, setPending] = useState<string | null>(null);
  const [error, setError] = useState<{
    message: string;
    base: Schemas["MatchDetail"];
    step?: number | null;
  }>();
  const busy = useRef(false);
  const step = shown.veto?.step ?? null;
  if (error && error.step === undefined && initial !== error.base) {
    setError({ ...error, step });
  } else if (error && error.step !== undefined && error.step !== step) {
    setError(undefined);
  }

  async function run(
    key: string,
    request: VetoRequest,
    optimistic?: Schemas["VetoView"],
  ) {
    if (busy.current) return;
    busy.current = true;
    setPending(key);
    setError(undefined);
    if (optimistic) preview(optimistic);
    const headers = hub?.originHeaders() ?? {};
    const sent = Date.now();
    const outcome = await callApi(request({ headers, step }));
    const received = Date.now();
    busy.current = false;
    setPending(null);
    if (outcome.ok) {
      noteServerTime(outcome.data.now, { sent, received });
      adopt(outcome.data);
    } else {
      preview(null);
      setError({
        message: errorMessage(outcome.code, outcome.params),
        base: initial,
      });
    }
    // O botão clicado some (o passo marcado) ou volta como outro elemento.
    focusIfLost(VETO_ID);
    // Em segundo plano: o resto da página e, no erro, a tela que ficou para
    // trás (o adversário agiu, o tempo acabou).
    router.refresh();
  }

  return { pending, error: error?.message, run };
}
