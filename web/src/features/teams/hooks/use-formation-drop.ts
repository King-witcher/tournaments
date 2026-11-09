import { useDroppable } from "@dnd-kit/core";

import { canMove, type Destination } from "../lib/formation";
import type { FormationBoardState } from "./use-formation-board";

/**
 * Um bloco que recebe jogadores arrastados (um time, a reserva, o time novo):
 * vai direto nas props dele. Só aceita o arrasto e fica destacado o bloco para
 * onde o jogador de agora pode ir; o andamento e o erro são os da última ação
 * nele.
 */
export function useFormationDrop(board: FormationBoardState, to: Destination) {
  const { drag, formation, target, pending, error } = board;
  const allowed = drag !== undefined && canMove(formation, drag.from, to);
  const { setNodeRef, isOver } = useDroppable({ id: to, disabled: !allowed });
  return {
    ref: setNodeRef,
    highlight: allowed && isOver,
    busy: pending && target === to,
    error: target === to ? error : undefined,
  };
}
