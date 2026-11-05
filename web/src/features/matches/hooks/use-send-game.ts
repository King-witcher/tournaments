import { startTransition, useActionState } from "react";

import { useRealtimeHub } from "@/features/realtime/hooks/use-realtime-hub";
import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { focusIfLost } from "@/shared/lib/focus";
import {
  gameInput,
  gameParams,
  gameRowId,
  type MatchTarget,
} from "../lib/match-target";
import { useMarkMine } from "./use-live-match";

type Sent = {
  game: Parameters<typeof gameInput>[0];
  /** Nota no histórico, só da organização. */
  comment: string;
};

/**
 * Envia o mapa `number`: `report` é o jogador (fica pendente), `organizer` a
 * organização (lança, resolve ou corrige, já confirmado, com a nota), `accept`
 * o capitão que aceita a contestação (já confirmado). Depois
 * de gravar, chama `onSent`, marca a mudança como de quem vê e recarrega a
 * página, com o foco no passo da partida; o erro volta traduzido. `pending`
 * dura até a página nova: `onSent` (que limpa e fecha o formulário) aparece
 * junto com ela. A aba recarrega sozinha, então o aviso da própria mudança não
 * volta pelo socket (`originHeaders`), e são uma recarga por envio, não duas.
 * `send` entra num `submitAction`.
 */
export function useSendGame(
  target: MatchTarget,
  number: number,
  mode: "report" | "organizer" | "accept",
  onSent?: () => void,
) {
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const hub = useRealtimeHub();
  const markMine = useMarkMine();
  const [error, dispatch, pending] = useActionState<string | undefined, Sent>(
    async (_previous, { game, comment }) => {
      const params = gameParams(target, number);
      const body = gameInput(game);
      const headers = hub?.originHeaders();
      const result =
        mode === "report"
          ? await callApi(
              api.POST(
                "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/report",
                { params, body, headers },
              ),
            )
          : mode === "accept"
            ? await callApi(
                api.POST(
                  "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/contest/accept",
                  { params, body, headers },
                ),
              )
            : await callApi(
                api.PUT(
                  "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}",
                  {
                    params,
                    body: { game: body, comment: comment.trim() || null },
                    headers,
                  },
                ),
              );
      if (!result.ok) return errorMessage(result.code, result.params);
      markMine();
      // Depois do `await`, só entra na transition (e no `pending`) o que roda
      // dentro de outra: o formulário some com a página nova, não antes.
      startTransition(() => {
        onSent?.();
        router.refresh();
      });
      // O formulário some: o foco vai para o passo da partida enviada (ou,
      // se os passos seguirem para outro, para ele: `MatchSteps`).
      focusIfLost(gameRowId(number));
      return undefined;
    },
    undefined,
  );
  return { error, pending, send: (sent: Sent) => dispatch(sent) };
}
