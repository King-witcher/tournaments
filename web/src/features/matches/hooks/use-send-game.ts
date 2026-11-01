import { useActionState } from "react";

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
 * organização (lança, resolve ou corrige, já confirmado, com a nota). Depois
 * de gravar, chama `onSent`, marca a mudança como de quem vê e recarrega a
 * página, com o foco no passo da partida; o erro volta traduzido.
 * `send` entra num `submitAction`.
 */
export function useSendGame(
  target: MatchTarget,
  number: number,
  mode: "report" | "organizer",
  onSent?: () => void,
) {
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const markMine = useMarkMine();
  const [error, dispatch, pending] = useActionState<string | undefined, Sent>(
    async (_previous, { game, comment }) => {
      const params = gameParams(target, number);
      const body = gameInput(game);
      const result =
        mode === "report"
          ? await callApi(
              api.POST(
                "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/report",
                { params, body },
              ),
            )
          : await callApi(
              api.PUT(
                "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}",
                {
                  params,
                  body: { game: body, comment: comment.trim() || null },
                },
              ),
            );
      if (!result.ok) return errorMessage(result.code, result.params);
      onSent?.();
      markMine();
      router.refresh();
      // O formulário some: o foco vai para o passo da partida enviada (ou,
      // se os passos seguirem para outro, para ele: `MatchSteps`).
      focusIfLost(gameRowId(number));
      return undefined;
    },
    undefined,
  );
  return { error, pending, send: (sent: Sent) => dispatch(sent) };
}
