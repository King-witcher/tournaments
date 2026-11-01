"use client";

import { api } from "@/shared/api/client";
import { serverTime } from "@/shared/lib/server-clock";
import { type MatchTarget, matchParams } from "../lib/match-target";
import { afterStep, type VetoChoice } from "../lib/veto-optimistic";
import { useLiveMatch } from "./use-live-match";

const VETO = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto";

/**
 * As ações do veto de uma série (pronto, forçar o começo, um passo,
 * desfazer, pedir e responder o pedido de desfazer), pelo
 * `LiveMatchProvider`: `pending` é a chave da que está no ar, para só o
 * botão dela girar (\`"ready"\`, \`"undo"\`...; num passo, \`"step"\`, e os
 * cartões ficam sem ação até a resposta). O passo aparece marcado na hora.
 * Cada ação diz a vez que a tela mostrava: se ela mudou antes do clique
 * chegar, a API recusa em vez de cair na vez seguinte.
 */
export function useVetoAction(target: MatchTarget) {
  const live = useLiveMatch();
  const params = matchParams(target);

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
    live.run(
      "step",
      ({ headers, step }) =>
        api.POST(`${VETO}/steps`, {
          params,
          headers,
          body: { ...choice, step },
        }),
      optimistic,
    );
  }

  return {
    pending: live.pending,
    step,
    ready: () =>
      live.run("ready", ({ headers }) =>
        api.POST(`${VETO}/ready`, { params, headers }),
      ),
    start: () =>
      live.run("start", ({ headers }) =>
        api.POST(`${VETO}/begin`, { params, headers }),
      ),
    undo: () =>
      live.run("undo", ({ headers, step }) =>
        api.POST(`${VETO}/undo`, { params, headers, body: { step } }),
      ),
    request: () =>
      live.run("request", ({ headers, step }) =>
        api.POST(`${VETO}/undo-request`, { params, headers, body: { step } }),
      ),
    answer: (accept: boolean) =>
      live.run(accept ? "accept" : "decline", ({ headers, step }) =>
        api.POST(`${VETO}/undo-request/answer`, {
          params,
          headers,
          body: { accept, step },
        }),
      ),
  };
}
