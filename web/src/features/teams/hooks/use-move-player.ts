import type { ApiResponse } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import type { Destination, Formation, Spot } from "../lib/formation";

/**
 * Mover um inscrito na formação, para a organização antes de iniciar: para
 * um time (`PUT .../teams/{team}/members/{user_id}`), para a reserva
 * (`DELETE` no time de onde sai) ou para um time novo com ele como capitão
 * (`POST .../teams`). Toda rota devolve a formação inteira, que vai para
 * `onDone`. `pending` e `error` (já traduzido) são desta instância: o arrastar
 * e o "Mover para…" de cada jogador mostram cada um o seu.
 */
export function useMovePlayer(tenant: string, tournament: string) {
  const { run, pending, error } = useApiAction();

  function request(
    userId: number,
    from: Spot,
    to: Destination,
  ): Promise<ApiResponse<Formation>> {
    const path = { slug: tenant, tournament };
    const member =
      "/tenants/{slug}/tournaments/{tournament}/teams/{team}/members/{user_id}";
    if (typeof to === "number") {
      return api.PUT(member, {
        params: { path: { ...path, team: to, user_id: userId } },
      });
    }
    // Para a reserva, só quem está num time (`canMove`).
    if (to === "reserve" && typeof from === "number") {
      return api.DELETE(member, {
        params: { path: { ...path, team: from, user_id: userId } },
      });
    }
    return api.POST("/tenants/{slug}/tournaments/{tournament}/teams", {
      params: { path },
      body: { user_id: userId },
    });
  }

  function move(
    userId: number,
    from: Spot,
    to: Destination,
    onDone: (formation: Formation) => void,
  ) {
    run(() => request(userId, from, to), onDone);
  }

  return { move, pending, error };
}
