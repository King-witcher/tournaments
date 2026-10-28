import { useSyncExternalStore } from "react";

import {
  setSoundEnabled,
  soundEnabled,
  subscribeSound,
} from "../lib/sound-preference";

/**
 * Se os avisos tocam som neste navegador, e a função que muda. Ligado no
 * servidor e até hidratar.
 */
export function useSoundEnabled() {
  const enabled = useSyncExternalStore(
    subscribeSound,
    soundEnabled,
    () => true,
  );
  return [enabled, setSoundEnabled] as const;
}
