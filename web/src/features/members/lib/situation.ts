import type { components } from "@/shared/api/schema";
import type { Status } from "@/shared/ui/status-tag";

/**
 * Cor da etiqueta pela situação de alguém num torneio: o mesmo mapa de "Seus
 * torneios" (`my-tournament-tile.tsx`), para a mesma situação ter a mesma
 * cor nas duas listas.
 */
export const SITUATION_STATUS: Record<
  components["schemas"]["Situation"],
  Status
> = {
  registered: "registration",
  check_in_pending: "check_in",
  checked_in: "confirmed",
  missed_check_in: "cancelled",
  playing: "in_progress",
  eliminated: "finished",
  withdrawn: "cancelled",
  champion: "confirmed",
};
