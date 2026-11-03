"use client";

import { useRef, useState } from "react";

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

export type VetoRequest<T> = (init: VetoRequestInit) => Promise<ApiResponse<T>>;

/** O que a resposta de uma ação do veto traz: a série de um torneio ou a sala. */
type VetoDetail = { now: string; veto?: Schemas["VetoView"] | null };

type Props<T> = {
  /** O que veio de fora (muda a cada recarga). */
  initial: T;
  /** O que a tela mostra, sem o passo marcado na hora. */
  shown: T;
  adopt: (detail: T) => void;
  preview: (veto: Schemas["VetoView"] | null) => void;
  /** Os cabeçalhos da ação (a aba que aplica a resposta sozinha). */
  headers: () => Record<string, string>;
  /** Depois de cada ação (`ok`: ela deu certo): busca de novo o que for preciso. */
  refresh: (ok: boolean) => void;
};

/**
 * As ações do veto, uma por vez para todos os tabuleiros (uma segunda, com
 * outra no ar, sairia com a vez velha): \`pending\` é a chave da que está no ar
 * e \`error\`, a mensagem da que falhou, que fica até a vez mudar depois da
 * recarga que o erro dispara (o "a vez mudou antes do seu clique" fala
 * justamente da vez nova), mesmo que o tabuleiro que agiu tenha saído da tela.
 * A resposta mede a hora da API pela ida e volta.
 */
export function useVetoRunner<T extends VetoDetail>({
  initial,
  shown,
  adopt,
  preview,
  headers,
  refresh,
}: Props<T>) {
  const errorMessage = useApiErrorMessage();
  const [pending, setPending] = useState<string | null>(null);
  const [error, setError] = useState<{
    message: string;
    base: T;
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
    request: VetoRequest<T>,
    optimistic?: Schemas["VetoView"],
  ) {
    if (busy.current) return;
    busy.current = true;
    setPending(key);
    setError(undefined);
    if (optimistic) preview(optimistic);
    const sent = Date.now();
    const outcome = await callApi(request({ headers: headers(), step }));
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
    // O botão clicado some (o passo marcado) ou volta como outro elemento. Sem
    // rolar: quem desceu a lista no celular continua onde estava.
    focusIfLost(VETO_ID, { preventScroll: true });
    // Em segundo plano: o resto da página e, no erro, a tela que ficou para
    // trás (o adversário agiu, o tempo acabou).
    refresh(outcome.ok);
  }

  return { pending, error: error?.message, run };
}
