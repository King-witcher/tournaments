import type { components } from "@/shared/api/schema";

/**
 * Como a situação de alguém num torneio aparece (`SituationMark`). Só a
 * exceção tem marca: quem ficou fora ou desistiu, em vermelho; quem ficou de
 * reserva sem time, em cinza; o empate que a organização ainda decide, com o
 * indicador de cuidado. O caminho esperado (inscrito, check-in feito, na
 * chave, eliminado, campeão…) é texto (`null`). É o mapa único: "Seus
 * torneios", o perfil do membro e "Sua situação" importam daqui.
 */
export const SITUATION_MARK: Record<
  components["schemas"]["Situation"],
  "cancelled" | "neutral" | "caution" | null
> = {
  registered: null,
  check_in_pending: null,
  checked_in: null,
  missed_check_in: "cancelled",
  reserve: "neutral",
  playing: null,
  tie_pending: "caution",
  eliminated: null,
  withdrawn: "cancelled",
  champion: null,
  awaiting_stage: null,
  stage_check_in_pending: null,
  substitute: null,
};

/** A situação tem marca (vai onde ficam as etiquetas, não no texto). */
export function hasSituationMark(
  situation: components["schemas"]["Situation"],
) {
  return SITUATION_MARK[situation] != null;
}
