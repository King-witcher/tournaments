import { useSyncExternalStore } from "react";

import { useRealtimeHub } from "./use-realtime-hub";

const nothing = () => () => {};
const closed = () => false;

/** Se o sininho do cabeçalho está aberto nesta aba. */
export function useBellOpen() {
  const hub = useRealtimeHub();
  return useSyncExternalStore(
    hub?.onBell ?? nothing,
    hub?.isBellOpen ?? closed,
    closed,
  );
}
