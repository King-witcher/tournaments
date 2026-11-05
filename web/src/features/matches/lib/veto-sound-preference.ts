import { storedSwitch } from "@/shared/lib/stored-switch";

/** Se o veto toca os sons neste navegador: ligado por padrão. */
export const vetoSound = storedSwitch("veto-sound");
