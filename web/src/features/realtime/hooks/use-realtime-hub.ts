import { createContext, useContext } from "react";

import type { RealtimeHub } from "../lib/realtime-hub";

/** A conexão da aba, do `RealtimeProvider` do layout. */
export const RealtimeContext = createContext<RealtimeHub | null>(null);

/** A conexão da aba; `null` fora do `RealtimeProvider`. */
export function useRealtimeHub() {
  return useContext(RealtimeContext);
}
