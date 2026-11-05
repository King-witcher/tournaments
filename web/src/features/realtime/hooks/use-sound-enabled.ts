import { useStoredSwitch } from "@/shared/hooks/use-stored-switch";
import { notificationSound } from "../lib/sound-preference";

/**
 * Se os avisos tocam som neste navegador, e a função que muda. Ligado no
 * servidor e até hidratar.
 */
export function useSoundEnabled() {
  return useStoredSwitch(notificationSound);
}
