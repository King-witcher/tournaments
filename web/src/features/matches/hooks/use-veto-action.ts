"use client";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { type MatchTarget, matchParams, VETO_ID } from "../lib/match-target";

/**
 * As ações do veto de uma partida (pronto, forçar o começo, um passo,
 * desfazer, pedir e responder o pedido de desfazer), com `pending` e o erro
 * já traduzido. Depois de agir, a página recarrega e o foco vai para o
 * bloco do veto, se o botão sumiu; com erro (a tela ficou para trás: o
 * adversário agiu, o tempo acabou), recarrega também.
 */
export function useVetoAction(target: MatchTarget) {
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const params = matchParams(target);
  const done = () => {
    router.refresh();
    focusIfLost(VETO_ID);
  };
  const stale = () => router.refresh();

  return {
    pending,
    error,
    ready: () =>
      run(
        () =>
          api.POST(
            "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto/ready",
            { params },
          ),
        done,
        stale,
      ),
    start: () =>
      run(
        () =>
          api.POST(
            "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto/begin",
            { params },
          ),
        done,
        stale,
      ),
    step: (choice: { map: string } | { champion: string }) =>
      run(
        () =>
          api.POST(
            "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto/steps",
            { params, body: choice },
          ),
        done,
        stale,
      ),
    undo: () =>
      run(
        () =>
          api.POST(
            "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto/undo",
            { params },
          ),
        done,
        stale,
      ),
    request: () =>
      run(
        () =>
          api.POST(
            "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto/undo-request",
            { params },
          ),
        done,
        stale,
      ),
    answer: (accept: boolean) =>
      run(
        () =>
          api.POST(
            "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto/undo-request/answer",
            { params, body: { accept } },
          ),
        done,
        stale,
      ),
  };
}
