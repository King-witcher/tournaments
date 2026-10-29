import type { components } from "@/shared/api/schema";
import type { Status } from "@/shared/ui/status-tag";

/**
 * Cor da etiqueta pelo significado da situação de alguém num torneio. É o
 * mapa único: "Seus torneios", o perfil do membro e "Sua situação" importam
 * daqui, para a mesma situação ter a mesma cor em todas as listas.
 */
export const SITUATION_STATUS: Record<
  components["schemas"]["Situation"],
  Status
> = {
  registered: "registration",
  check_in_pending: "check_in",
  checked_in: "confirmed",
  missed_check_in: "cancelled",
  reserve: "neutral",
  playing: "in_progress",
  eliminated: "finished",
  withdrawn: "cancelled",
  champion: "confirmed",
};
