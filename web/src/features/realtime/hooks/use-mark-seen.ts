import { useCallback } from "react";

import { api } from "@/shared/api/client";
import { useRealtimeHub } from "./use-realtime-hub";

/**
 * Devolve a função que marca como vistas no sininho as notificações destes
 * toasts (o `id` de cada um). A contagem volta na resposta e, nas outras
 * abas, pelo WebSocket. Estável.
 */
export function useMarkSeen() {
  const hub = useRealtimeHub();
  return useCallback(
    (ids: string[]) => {
      api
        .POST("/me/notifications/read", { body: { ids: ids.map(Number) } })
        .then(({ data }) => {
          if (data) hub?.setUnread(data.unread);
        })
        // Sem a resposta, continuam novas no sininho.
        .catch(() => {});
    },
    [hub],
  );
}
