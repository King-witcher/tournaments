import type { components } from "@/shared/api/schema";
import type { Status } from "@/shared/ui/status-tag";

/** Cor da etiqueta pelo significado da situação de quem joga (a mesma de "Seus torneios"). */
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
