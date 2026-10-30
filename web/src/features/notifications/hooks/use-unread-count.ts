import { useSyncExternalStore } from "react";

import { useRealtimeHub } from "@/features/realtime/hooks/use-realtime-hub";

const nothing = () => () => {};
const none = () => undefined;

/**
 * Quantas notificações faltam ver, da conexão da aba (chega quando o socket
 * abre e muda a cada aviso). `undefined` antes disso e no servidor.
 */
export function useUnreadCount() {
  const hub = useRealtimeHub();
  return useSyncExternalStore(
    hub?.onUnread ?? nothing,
    hub?.getUnread ?? none,
    none,
  );
}
