import { storedSwitch } from "@/shared/lib/stored-switch";

/** Se os avisos com o site aberto tocam som neste navegador: ligado por padrão. */
export const notificationSound = storedSwitch("notification-sound");

export const soundEnabled = notificationSound.get;
