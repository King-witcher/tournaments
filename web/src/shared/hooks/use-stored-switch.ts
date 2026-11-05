import { useSyncExternalStore } from "react";

import type { StoredSwitch } from "@/shared/lib/stored-switch";

/** A escolha de um `storedSwitch` e a função que muda; ligada no servidor e até hidratar. */
export function useStoredSwitch(stored: StoredSwitch) {
  const enabled = useSyncExternalStore(
    stored.subscribe,
    stored.get,
    () => true,
  );
  return [enabled, stored.set] as const;
}
