import { api } from "@/shared/api/client";
import { serverTime } from "@/shared/lib/server-clock";
import type { LiveMatch } from "../hooks/use-live-match";
import type { VetoActions } from "../hooks/use-veto-board";
import { type MatchTarget, matchParams } from "./match-target";
import { afterStep, type VetoChoice } from "./veto-optimistic";

const VETO = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto";

/**
 * As ações do veto de uma série (pronto e desfazê-lo, forçar o começo, um passo,
 * desfazer, pedir e responder o pedido de desfazer): o `LiveMatchProvider` as põe no
 * `VetoBoard`. O passo aparece marcado na hora. Cada ação diz a vez que a tela
 * mostrava: se ela mudou antes do clique chegar, a API recusa em vez de cair
 * na vez seguinte. `live` dá a série como a tela a mostra na hora do clique,
 * para as ações serem as mesmas entre renderizações.
 */
export function matchVetoActions(
  target: MatchTarget,
  live: () => Pick<
    LiveMatch,
    "detail" | "viewerId" | "bansPerSeries" | "vetoOrder"
  >,
  run: LiveMatch["run"],
): VetoActions {
  const params = matchParams(target);

  function step(choice: VetoChoice) {
    const { detail, viewerId, bansPerSeries, vetoOrder } = live();
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
    run(
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
    step,
    ready: () =>
      run("ready", ({ headers }) =>
        api.POST(`${VETO}/ready`, { params, headers }),
      ),
    unready: () =>
      run("unready", ({ headers }) =>
        api.DELETE(`${VETO}/ready`, { params, headers }),
      ),
    start: () =>
      run("start", ({ headers }) =>
        api.POST(`${VETO}/begin`, { params, headers }),
      ),
    undo: () =>
      run("undo", ({ headers, step }) =>
        api.POST(`${VETO}/undo`, { params, headers, body: { step } }),
      ),
    request: () =>
      run("request", ({ headers, step }) =>
        api.POST(`${VETO}/undo-request`, { params, headers, body: { step } }),
      ),
    answer: (accept: boolean) =>
      run(accept ? "accept" : "decline", ({ headers, step }) =>
        api.POST(`${VETO}/undo-request/answer`, {
          params,
          headers,
          body: { accept, step },
        }),
      ),
  };
}
