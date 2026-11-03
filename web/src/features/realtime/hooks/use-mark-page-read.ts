import { useCallback } from "react";

import { api } from "@/shared/api/client";
import { useRealtimeHub } from "./use-realtime-hub";

/**
 * Devolve a função que marca como lidos os avisos de uma página (o caminho sem
 * o idioma, exato). A contagem volta na resposta e, nas outras abas, pelo
 * WebSocket. Estável.
 */
export function useMarkPageRead() {
  const hub = useRealtimeHub();
  return useCallback(
    (path: string) => {
      api
        .POST("/me/notifications/read", { body: { path } })
        .then(({ data }) => {
          if (data) hub?.setUnread(data.unread);
        })
        // Sem a resposta, continuam não lidos no sininho.
        .catch(() => {});
    },
    [hub],
  );
}
