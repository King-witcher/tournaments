"use client";

import { useRef, useState } from "react";

import { useRealtimeHub } from "@/features/realtime/hooks/use-realtime-hub";
import { useRouter } from "@/i18n/navigation";
import { type ApiResponse, callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { focusIfLost } from "@/shared/lib/focus";
import { noteServerTime, serverTime } from "@/shared/lib/server-clock";
import { type MatchTarget, matchParams, VETO_ID } from "../lib/match-target";
import { afterStep, type VetoChoice } from "../lib/veto-optimistic";
import { useLiveMatch } from "./use-live-match";

type Schemas = components["schemas"];

/**
 * As ações do veto de uma série (pronto, forçar o começo, um passo,
 * desfazer, pedir e responder o pedido de desfazer). `pending` é a chave da
 * ação em andamento (`"ready"`, `"step:awoken"`...), para só o botão
 * clicado girar; `error`, a mensagem já traduzida. O passo aparece na hora
 * (marcado e com a vez passada) e a resposta da API, que traz a série
 * inteira, atualiza a área dos passos; a recarga da página corre em segundo
 * plano para o resto (cabeçalho, placar, partidas), sem prender o botão, e o
 * eco da ação pelo socket não volta a esta aba (`originHeaders`). Cada ação
 * diz a vez que a tela mostrava (`step`): se ela mudou antes do clique
 * chegar, a API recusa em vez de cair na vez seguinte. No erro, o passo
 * marcado volta atrás.
 */
export function useVetoAction(target: MatchTarget) {
  const router = useRouter();
  const hub = useRealtimeHub();
  const live = useLiveMatch();
  const errorMessage = useApiErrorMessage();
  const [pending, setPending] = useState<string | null>(null);
  const [error, setError] = useState<string>();
  const busy = useRef(false);
  const params = matchParams(target);

  async function run(
    key: string,
    request: (init: {
      headers: Record<string, string>;
      step: number | null;
    }) => Promise<ApiResponse<Schemas["MatchDetail"]>>,
    optimistic?: Schemas["VetoView"],
  ) {
    if (busy.current) return;
    busy.current = true;
    setPending(key);
    setError(undefined);
    const step = live.detail.veto?.step ?? null;
    if (optimistic) live.preview(optimistic);
    const headers = hub?.originHeaders() ?? {};
    const outcome = await callApi(request({ headers, step }));
    busy.current = false;
    setPending(null);
    if (outcome.ok) {
      noteServerTime(outcome.data.now);
      live.adopt(outcome.data);
    } else {
      live.preview(null);
      setError(errorMessage(outcome.code, outcome.params));
    }
    // O botão clicado some (o passo marcado) ou volta como outro elemento.
    focusIfLost(VETO_ID);
    // No erro também: a tela pode ter ficado para trás (o adversário agiu,
    // o tempo acabou).
    router.refresh();
  }

  function step(choice: VetoChoice) {
    const { detail, viewerId, bansPerSeries, vetoOrder } = live;
    const mine = detail.viewer_side;
    const member = [
      ...(detail.a?.members ?? []),
      ...(detail.b?.members ?? []),
    ].find((m) => m.user_id === viewerId);
    const optimistic =
      detail.veto && mine
        ? afterStep(detail.veto, choice, {
            mine,
            bestOf: detail.summary.best_of,
            bansPerSeries,
            order: vetoOrder,
            viewer: member
              ? { id: member.user_id, nickname: member.nickname }
              : null,
            now: serverTime(),
          })
        : undefined;
    const key = "map" in choice ? choice.map : choice.champion;
    run(
      `step:${key}`,
      ({ headers, step }) =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto/steps",
          { params, headers, body: { ...choice, step } },
        ),
      optimistic,
    );
  }

  return {
    pending,
    error,
    step,
    ready: () =>
      run("ready", ({ headers }) =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto/ready",
          { params, headers },
        ),
      ),
    start: () =>
      run("start", ({ headers }) =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto/begin",
          { params, headers },
        ),
      ),
    undo: () =>
      run("undo", ({ headers, step }) =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto/undo",
          { params, headers, body: { step } },
        ),
      ),
    request: () =>
      run("request", ({ headers, step }) =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto/undo-request",
          { params, headers, body: { step } },
        ),
      ),
    answer: (accept: boolean) =>
      run(accept ? "accept" : "decline", ({ headers, step }) =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto/undo-request/answer",
          { params, headers, body: { accept, step } },
        ),
      ),
  };
}
